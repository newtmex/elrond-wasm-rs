use elrond_wasm::abi::{ContractAbi, EndpointAbi};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
};

use super::{MultiContractConfigSerde, OutputContract, OutputContractConfig, OutputContractSerde};

#[derive(Default)]
struct OutputContractBuilder {
    pub config_id: String,
    pub config_name: String,
    pub external_view: bool,
    pub add_unlabelled: bool,
    pub add_labels: BTreeSet<String>,
    pub constructors: Vec<EndpointAbi>,
    pub endpoint_names: HashSet<String>,
    pub endpoints: Vec<EndpointAbi>,
}

impl OutputContractBuilder {
    fn new(id: String) -> Self {
        OutputContractBuilder {
            config_id: id.clone(),
            config_name: id,
            external_view: false, // if unspecified, it should be considered false
            ..Default::default()
        }
    }

    fn map_from_config(kvp: (&String, &OutputContractSerde)) -> (String, OutputContractBuilder) {
        let (config_name, cms) = kvp;
        let external_view = cms.external_view.unwrap_or_default();
        let mut constructors = Vec::new();
        if external_view {
            constructors
                .push(elrond_wasm::external_view_contract::external_view_contract_constructor_abi())
        }
        (
            config_name.clone(),
            OutputContractBuilder {
                config_id: config_name.clone(),
                config_name: cms.name.clone().unwrap_or_default(),
                external_view,
                add_unlabelled: cms.add_unlabelled.unwrap_or_default(),
                add_labels: cms.add_labels.iter().cloned().collect(),
                constructors,
                ..Default::default()
            },
        )
    }

    fn wasm_name(&self) -> &String {
        if !self.config_name.is_empty() {
            &self.config_name
        } else {
            &self.config_id
        }
    }

    /// Keeps endpoints unique, while retaining their order.
    fn add_constructor(&mut self, constructor_abi: &EndpointAbi) {
        assert!(
            self.constructors.is_empty(),
            "More than one constructor specified for contract {}",
            &self.config_name
        );
        self.constructors.push(constructor_abi.clone());
    }

    /// Keeps endpoints unique, while retaining their order.
    fn add_endpoint(&mut self, endpoint_abi: &EndpointAbi) {
        if !self.endpoint_names.contains(endpoint_abi.name) {
            self.endpoint_names.insert(endpoint_abi.name.to_string());
            self.endpoints.push(endpoint_abi.clone());
        }
    }
}

fn process_labels_for_contracts(
    contract_builders: &mut HashMap<String, OutputContractBuilder>,
    labels_for_contracts: &HashMap<String, Vec<String>>,
) {
    for (label, targets) in labels_for_contracts {
        for target in targets {
            contract_builders
                .entry(target.clone())
                .or_insert_with(|| OutputContractBuilder::new(target.clone()))
                .add_labels
                .insert(label.clone());
        }
    }
}

fn endpoint_unlabelled(endpoint_abi: &EndpointAbi) -> bool {
    endpoint_abi.labels.is_empty()
}

fn endpoint_matches_labels(endpoint_abi: &EndpointAbi, labels: &BTreeSet<String>) -> bool {
    endpoint_abi
        .labels
        .iter()
        .any(|&endpoint_label| labels.contains(endpoint_label))
}

fn collect_unlabelled_endpoints(
    contract_builders: &mut HashMap<String, OutputContractBuilder>,
    original_abi: &ContractAbi,
) {
    for builder in contract_builders.values_mut() {
        if builder.add_unlabelled {
            for constructor_abi in &original_abi.constructors {
                if endpoint_unlabelled(constructor_abi) {
                    builder.add_constructor(constructor_abi);
                }
            }
            for endpoint_abi in &original_abi.endpoints {
                if endpoint_unlabelled(endpoint_abi) {
                    builder.add_endpoint(endpoint_abi);
                }
            }
        }
    }
}

fn collect_labelled_endpoints(
    contract_builders: &mut HashMap<String, OutputContractBuilder>,
    original_abi: &ContractAbi,
) {
    for builder in contract_builders.values_mut() {
        for constructor_abi in &original_abi.constructors {
            if endpoint_matches_labels(constructor_abi, &builder.add_labels) {
                builder.add_constructor(constructor_abi);
            }
        }
        for endpoint_abi in &original_abi.endpoints {
            if endpoint_matches_labels(endpoint_abi, &builder.add_labels) {
                builder.add_endpoint(endpoint_abi);
            }
        }
    }
}

fn build_contract_abi(builder: OutputContractBuilder, original_abi: &ContractAbi) -> ContractAbi {
    ContractAbi {
        build_info: original_abi.build_info.clone(),
        docs: original_abi.docs,
        name: original_abi.name,
        constructors: builder.constructors,
        endpoints: builder.endpoints,
        events: original_abi.events.clone(),
        has_callback: !builder.external_view && original_abi.has_callback,
        type_descriptions: original_abi.type_descriptions.clone(),
    }
}

fn build_contract(builder: OutputContractBuilder, original_abi: &ContractAbi) -> OutputContract {
    let name = builder.wasm_name().clone();
    OutputContract {
        main: false,
        external_view: builder.external_view,
        config_name: builder.config_id.clone(),
        public_name: name,
        abi: build_contract_abi(builder, original_abi),
        cargo_toml_contents_cache: None,
    }
}

fn set_main_contract_flag(
    contracts: &mut [OutputContract],
    default_contract_config_name_opt: &Option<String>,
) {
    if let Some(default_contract_config_name) = default_contract_config_name_opt {
        for contract in contracts.iter_mut() {
            if &contract.config_name == default_contract_config_name {
                contract.main = true;
                return;
            }
        }

        panic!(
            "Could not find default contract '{}' among the output contracts. Available contracts are: {:?}",
            default_contract_config_name,
            contracts.iter().map(|contract| &contract.config_name).collect::<Vec<_>>(),
        )
    } else {
        let first_contract = contracts.get_mut(0).unwrap_or_else(|| {
            panic!("Cannot set default contract because no optput contract was specified.")
        });
        first_contract.main = true;
    }
}

impl OutputContractConfig {
    pub fn load_from_config(config: &MultiContractConfigSerde, original_abi: &ContractAbi) -> Self {
        let mut contract_builders: HashMap<String, OutputContractBuilder> = config
            .contracts
            .iter()
            .map(OutputContractBuilder::map_from_config)
            .collect();
        collect_unlabelled_endpoints(&mut contract_builders, original_abi);
        collect_labelled_endpoints(&mut contract_builders, original_abi);
        process_labels_for_contracts(&mut contract_builders, &config.labels_for_contracts);
        let mut contracts: Vec<OutputContract> = contract_builders
            .into_values()
            .map(|builder| build_contract(builder, original_abi))
            .collect();
        set_main_contract_flag(&mut contracts, &config.settings.main);
        OutputContractConfig {
            default_contract_config_name: config.settings.main.clone().unwrap_or_default(),
            contracts,
        }
    }

    pub fn default_config(original_abi: &ContractAbi) -> Self {
        let default_contract_config_name = original_abi.build_info.contract_crate.name.to_string();
        OutputContractConfig {
            default_contract_config_name: default_contract_config_name.clone(),
            contracts: vec![OutputContract {
                main: true,
                external_view: false,
                config_name: default_contract_config_name.clone(),
                public_name: default_contract_config_name,
                abi: original_abi.clone(),
                cargo_toml_contents_cache: None,
            }],
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P, original_abi: &ContractAbi) -> Option<Self> {
        match fs::read_to_string(path.as_ref()) {
            Ok(s) => {
                let config_serde: MultiContractConfigSerde = toml::from_str(s.as_str())
                    .unwrap_or_else(|error| panic!("error parsing multicontract.toml: {}", error));
                Some(Self::load_from_config(&config_serde, original_abi))
            },
            Err(_) => None,
        }
    }

    pub fn load_from_file_or_default<P: AsRef<Path>>(path: P, original_abi: &ContractAbi) -> Self {
        Self::load_from_file(path, original_abi)
            .unwrap_or_else(|| Self::default_config(original_abi))
    }
}
