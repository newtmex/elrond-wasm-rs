#![no_std]

imports!();

mod lottery_info;
mod random;
mod status;

use lottery_info::LotteryInfo;
use random::Random;
use status::Status;

const PERCENTAGE_TOTAL: u16 = 100;
const THIRTY_DAYS_IN_SECONDS: u64 = 60 * 60 * 24 * 30;

#[elrond_wasm_derive::callable(Erc20Proxy)]
pub trait Erc20 {
	#[callback(transfer_from_callback)]
	fn transferFrom(
		&self,
		sender: &Address,
		recipient: &Address,
		amount: BigUint,
		#[callback_arg] cb_lottery_name: Vec<u8>,
		#[callback_arg] cb_sender: &Address,
	);

	#[callback(distribute_prizes_callback)]
	fn transfer(&self, to: &Address, amount: BigUint, #[callback_arg] cb_lottery_name: Vec<u8>);
}

#[elrond_wasm_derive::contract(LotteryImpl)]
pub trait Lottery {
	#[init]
	fn init(&self, erc20_contract_address: Address) {
		self.set_erc20_contract_address(&erc20_contract_address);
	}

	#[endpoint]
	fn start(
		&self,
		lottery_name: Vec<u8>,
		ticket_price: BigUint,
		opt_total_tickets: Option<u32>,
		opt_deadline: Option<u64>,
		opt_max_entries_per_user: Option<u32>,
		opt_prize_distribution: Option<Vec<u8>>,
		opt_whitelist: Option<Vec<Address>>,
	) -> SCResult<()> {
		self.start_lottery(
			lottery_name,
			ticket_price,
			opt_total_tickets,
			opt_deadline,
			opt_max_entries_per_user,
			opt_prize_distribution,
			opt_whitelist,
		)
	}

	#[endpoint(createLotteryPool)]
	fn create_lottery_pool(
		&self,
		lottery_name: Vec<u8>,
		ticket_price: BigUint,
		opt_total_tickets: Option<u32>,
		opt_deadline: Option<u64>,
		opt_max_entries_per_user: Option<u32>,
		opt_prize_distribution: Option<Vec<u8>>,
		opt_whitelist: Option<Vec<Address>>,
	) -> SCResult<()> {
		self.start_lottery(
			lottery_name,
			ticket_price,
			opt_total_tickets,
			opt_deadline,
			opt_max_entries_per_user,
			opt_prize_distribution,
			opt_whitelist,
		)
	}

	fn start_lottery(
		&self,
		lottery_name: Vec<u8>,
		ticket_price: BigUint,
		opt_total_tickets: Option<u32>,
		opt_deadline: Option<u64>,
		opt_max_entries_per_user: Option<u32>,
		opt_prize_distribution: Option<Vec<u8>>,
		opt_whitelist: Option<Vec<Address>>,
	) -> SCResult<()> {
		require!(!lottery_name.is_empty(), "Name can't be empty!");

		let timestamp = self.get_block_timestamp();

		let total_tickets = opt_total_tickets.unwrap_or(u32::MAX);
		let deadline = opt_deadline.unwrap_or_else(|| timestamp + THIRTY_DAYS_IN_SECONDS);
		let max_entries_per_user = opt_max_entries_per_user.unwrap_or(u32::MAX);
		let prize_distribution =
			opt_prize_distribution.unwrap_or_else(|| [PERCENTAGE_TOTAL as u8].to_vec());
		let whitelist = opt_whitelist.unwrap_or(Vec::new());

		require!(
			self.status(lottery_name.clone()) == Status::Inactive,
			"Lottery is already active!"
		);

		require!(ticket_price > 0, "Ticket price must be higher than 0!");

		require!(
			total_tickets > 0,
			"Must have more than 0 tickets available!"
		);

		require!(deadline > timestamp, "Deadline can't be in the past!");

		require!(
			deadline <= timestamp + THIRTY_DAYS_IN_SECONDS,
			"Deadline can't be later than 30 days from now!"
		);

		require!(
			max_entries_per_user > 0,
			"Must have more than 0 max entries per user!"
		);

		require!(
			self.sum_array(&prize_distribution) == PERCENTAGE_TOTAL,
			"Prize distribution must add up to exactly 100(%)!"
		);

		let info = LotteryInfo {
			ticket_price,
			tickets_left: total_tickets,
			deadline,
			max_entries_per_user,
			prize_distribution,
			whitelist,
			current_ticket_number: 0u32,
			prize_pool: BigUint::zero(),
			queued_tickets: 0u32,
		};

		self.set_lottery_exists(&lottery_name, true);
		self.set_lottery_info(&lottery_name, &info);

		Ok(())
	}

	#[endpoint]
	fn buy_ticket(&self, lottery_name: Vec<u8>, token_amount: BigUint) -> SCResult<()> {
		match self.status(lottery_name.clone()) {
			Status::Inactive => sc_error!("Lottery is currently inactive."),
			Status::Running => self.update_after_buy_ticket(&lottery_name, token_amount),
			Status::Ended => {
				sc_error!("Lottery entry period has ended! Awaiting winner announcement.")
			},
			Status::DistributingPrizes => {
				sc_error!("Prizes are currently being distributed. Can't buy tickets!")
			},
		}
	}

	#[endpoint]
	fn determine_winner(&self, lottery_name: Vec<u8>) -> SCResult<()> {
		match self.status(lottery_name.clone()) {
			Status::Inactive => sc_error!("Lottery is inactive!"),
			Status::Running => sc_error!("Lottery is still running!"),
			Status::Ended => {
				let info = self.get_mut_lottery_info(&lottery_name);

				if info.queued_tickets > 0 {
					return sc_error!(
						"There are still tickets being processed. Please try again later."
					);
				}

				self.distribute_prizes(&lottery_name);

				Ok(())
			},
			Status::DistributingPrizes => sc_error!("Prizes are currently being distributed!"),
		}
	}

	#[view]
	fn status(&self, lottery_name: Vec<u8>) -> Status {
		let exists = self.get_lottery_exists(&lottery_name);

		if !exists {
			return Status::Inactive;
		}

		let prev_winners = self.get_prev_winners(&lottery_name);

		if prev_winners.len() > 0 {
			return Status::DistributingPrizes;
		}

		let info = self.get_mut_lottery_info(&lottery_name);

		if self.get_block_timestamp() > info.deadline || info.tickets_left == 0 {
			return Status::Ended;
		}

		return Status::Running;
	}

	fn update_after_buy_ticket(
		&self,
		lottery_name: &Vec<u8>,
		token_amount: BigUint,
	) -> SCResult<()> {
		let info = self.get_mut_lottery_info(&lottery_name);
		let caller = self.get_caller();

		require!(
			info.whitelist.is_empty() || info.whitelist.contains(&caller),
			"You are not allowed to participate in this lottery!"
		);

		require!(token_amount == info.ticket_price, "Wrong ticket fee!");

		let entries = self.get_mut_number_of_entries_for_user(&lottery_name, &caller);

		require!(
			*entries < info.max_entries_per_user,
			"Ticket limit exceeded for this lottery!"
		);

		// reserve the ticket, but don't update the other fields yet.
		self.reserve_ticket(lottery_name);

		let erc20_address = self.get_erc20_contract_address();
		let lottery_contract_address = self.get_sc_address();
		let erc20_proxy = contract_proxy!(self, &erc20_address, Erc20);
		erc20_proxy.transferFrom(
			&caller,
			&lottery_contract_address,
			token_amount,
			lottery_name.clone(),
			&caller,
		);

		Ok(())
	}

	fn reserve_ticket(&self, lottery_name: &Vec<u8>) {
		let mut info = self.get_mut_lottery_info(&lottery_name);

		info.tickets_left -= 1;
		info.queued_tickets += 1;
	}

	fn reduce_prize_pool(&self, lottery_name: &Vec<u8>, value: BigUint) {
		let mut info = self.get_mut_lottery_info(&lottery_name);
		info.prize_pool -= value;
	}

	fn distribute_prizes(&self, lottery_name: &Vec<u8>) {
		let info = self.get_mut_lottery_info(&lottery_name);

		let total_tickets = info.current_ticket_number;
		let total_winning_tickets = info.prize_distribution.len();
		let mut prev_winners = self.get_prev_winners(&lottery_name);
		let prev_winners_count = prev_winners.len();
		let winners_left = total_winning_tickets - prev_winners_count;

		if winners_left == 0 {
			self.clear_storage(&lottery_name);

			return;
		}

		let last_winning_ticket_index: usize;

		// less tickets purchased than total winning tickets
		if total_tickets < total_winning_tickets as u32 {
			last_winning_ticket_index = (total_tickets - 1) as usize;
		} else {
			last_winning_ticket_index = info.prize_distribution.len() - 1;
		}

		let current_winning_ticket_index = last_winning_ticket_index - prev_winners_count;
		let winning_ticket_id = self.get_random_winning_ticket_id(&prev_winners, total_tickets);

		let winner_address = self.get_ticket_holder(&lottery_name, winning_ticket_id);
		let prize: BigUint;

		if current_winning_ticket_index != 0 {
			prize = BigUint::from(info.prize_distribution[current_winning_ticket_index] as u32)
				* info.prize_pool.clone()
				/ BigUint::from(PERCENTAGE_TOTAL as u32);
		} else {
			prize = info.prize_pool.clone();
		}

		self.reduce_prize_pool(lottery_name, prize.clone());

		prev_winners.push(winning_ticket_id);
		self.set_prev_winners(lottery_name, &prev_winners);

		let erc20_address = self.get_erc20_contract_address();
		let erc20_proxy = contract_proxy!(self, &erc20_address, Erc20);

		erc20_proxy.transfer(&winner_address, prize, lottery_name.clone());
	}

	fn get_random_winning_ticket_id(&self, prev_winners: &Vec<u32>, total_tickets: u32) -> u32 {
		let seed = self.get_block_random_seed();
		let mut rand = Random::new(*seed);

		loop {
			let winner = rand.next() % total_tickets;

			if !prev_winners.contains(&winner) {
				return winner;
			}
		}
	}

	fn clear_storage(&self, lottery_name: &Vec<u8>) {
		let name_len_vec = &(lottery_name.len() as u32).to_be_bytes().to_vec();
		let temp = [&name_len_vec[..], &lottery_name[..]].concat(); // "temporary value dropped" otherwise
		let appended_name_in_key = temp.as_slice();

		let info = self.get_mut_lottery_info(lottery_name);

		for i in 0..info.current_ticket_number {
			let addr = self.get_ticket_holder(lottery_name, i);
			let key_ticket_holder = [
				"ticketHolder".as_bytes(),
				appended_name_in_key,
				&i.to_be_bytes(),
			]
			.concat();
			let key_number_of_entries = [
				"numberOfEntriesForUser".as_bytes(),
				appended_name_in_key,
				addr.as_bytes(),
			]
			.concat();

			self.storage_store_slice_u8(&key_ticket_holder, &[0u8; 0]);
			self.storage_store_slice_u8(&key_number_of_entries, &[0u8; 0]);
		}

		self.storage_store_slice_u8(
			&["lotteryExists".as_bytes(), appended_name_in_key].concat(),
			&[0u8; 0],
		);
		self.storage_store_slice_u8(
			&["lotteryInfo".as_bytes(), appended_name_in_key].concat(),
			&[0u8; 0],
		);
		self.storage_store_slice_u8(
			&["previousWinners".as_bytes(), appended_name_in_key].concat(),
			&[0u8; 0],
		);
	}

	fn sum_array(&self, array: &[u8]) -> u16 {
		let mut sum = 0u16; // u16 to protect against overflow

		for i in 0..array.len() {
			sum += array[i] as u16;
		}

		return sum;
	}

	#[callback]
	fn transfer_from_callback(
		&self,
		result: AsyncCallResult<()>,
		#[callback_arg] cb_lottery_name: Vec<u8>,
		#[callback_arg] cb_sender: Address,
	) {
		let mut info = self.get_mut_lottery_info(&cb_lottery_name);

		match result {
			AsyncCallResult::Ok(()) => {
				let mut entries =
					self.get_mut_number_of_entries_for_user(&cb_lottery_name, &cb_sender);

				self.set_ticket_holder(&cb_lottery_name, info.current_ticket_number, &cb_sender);

				*entries += 1;
				info.current_ticket_number += 1;

				let ticket_price = info.ticket_price.clone();
				info.prize_pool += ticket_price;
			},
			AsyncCallResult::Err(_) => {
				// payment error, return ticket to pool
				info.tickets_left += 1;
			},
		}

		info.queued_tickets -= 1;
	}

	#[callback]
	fn distribute_prizes_callback(
		&self,
		result: AsyncCallResult<()>,
		#[callback_arg] cb_lottery_name: Vec<u8>,
	) {
		match result {
			AsyncCallResult::Ok(()) => {
				self.distribute_prizes(&cb_lottery_name);
			},
			AsyncCallResult::Err(_) => {
				// nothing we can do if an error occurs in the erc20 contract
			},
		}
	}

	#[storage_set("lotteryExists")]
	fn set_lottery_exists(&self, lottery_name: &[u8], exists: bool);

	#[view(lotteryExists)]
	#[storage_get("lotteryExists")]
	fn get_lottery_exists(&self, lottery_name: &Vec<u8>) -> bool;

	#[storage_set("lotteryInfo")]
	fn set_lottery_info(&self, lottery_name: &[u8], lottery_info: &LotteryInfo<BigUint>);

	#[view(lotteryInfo)]
	#[storage_get_mut("lotteryInfo")]
	fn get_mut_lottery_info(&self, lottery_name: &Vec<u8>) -> mut_storage!(LotteryInfo<BigUint>);

	#[storage_set("ticketHolder")]
	fn set_ticket_holder(&self, lottery_name: &[u8], ticket_id: u32, ticket_holder: &Address);

	#[storage_get("ticketHolder")]
	fn get_ticket_holder(&self, lottery_name: &[u8], ticket_id: u32) -> Address;

	#[storage_get_mut("numberOfEntriesForUser")]
	fn get_mut_number_of_entries_for_user(
		&self,
		lottery_name: &[u8],
		user: &Address,
	) -> mut_storage!(u32);

	#[storage_set("erc20_contract_address")]
	fn set_erc20_contract_address(&self, address: &Address);

	#[view(erc20ContractAddress)]
	#[storage_get("erc20_contract_address")]
	fn get_erc20_contract_address(&self) -> Address;

	// temporary storage between "determine_winner" proxy callbacks

	#[storage_get("previousWinners")]
	fn get_prev_winners(&self, lottery_name: &[u8]) -> Vec<u32>;

	#[storage_set("previousWinners")]
	fn set_prev_winners(&self, lottery_name: &[u8], prev_winners: &Vec<u32>);
}
