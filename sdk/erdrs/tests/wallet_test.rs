use bip39::Mnemonic;

use mx_sdk_erdrs::{
    crypto::public_key::PublicKey, data::address::Address, interactors::wallet::Wallet,
};

#[test]
fn test_private_key_from_mnemonic() {
    let mnemonic: Mnemonic = Mnemonic::parse_normalized("acid twice post genre topic observe valid viable gesture fortune funny dawn around blood enemy page update reduce decline van bundle zebra rookie real").unwrap();

    let private_key = Wallet::get_private_key_from_mnemonic(mnemonic.clone(), 0, 0);
    let public_key = PublicKey::from(&private_key);
    let address = Address::from(&public_key);
    assert_eq!(
        "0b7966138e80b8f3bb64046f56aea4250fd7bacad6ed214165cea6767fd0bc2c",
        private_key.to_string()
    );
    assert_eq!(
        "dfefe0453840e5903f2bd519de9b0ed6e9621e57e28ba0b4c1b15115091dd72f",
        public_key.to_string()
    );
    assert_eq!(
        "erd1mlh7q3fcgrjeq0et65vaaxcw6m5ky8jhu296pdxpk9g32zga6uhsemxx2a",
        address.to_string()
    );

    let private_key = Wallet::get_private_key_from_mnemonic(mnemonic, 0, 1);
    let public_key = PublicKey::from(&private_key);
    let address = Address::from(&public_key);
    assert_eq!(
        "1648ad209d6b157a289884933e3bb30f161ec7113221ec16f87c3578b05830b0",
        private_key.to_string()
    );
    assert_eq!(
        "0ef8c4e4906e724d067c09466c3794b3c5add9842abd215efbdc306b3db9b5ac",
        public_key.to_string()
    );
    assert_eq!(
        "erd1pmuvfeysdeey6pnup9rxcdu5k0z6mkvy927jzhhmmscxk0dekkkq3wupvu",
        address.to_string()
    );
}

#[test]
fn test_load_from_pem() {
    let wallet = Wallet::from_pem_file("tests/alice.pem").unwrap();
    let addr = wallet.address();
    assert_eq!(
        addr.to_bech32_string().unwrap(),
        "erd1qyu5wthldzr8wx5c9ucg8kjagg0jfs53s8nr3zpz3hypefsdd8ssycr6th"
    );
}
