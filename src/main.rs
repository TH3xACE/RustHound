pub mod modules;
pub mod enums;
pub mod json;

pub mod args;
pub mod banner;
pub mod errors;
pub mod ldap;

use log::{info,trace,error};
use std::collections::HashMap;

use crate::errors::Result;
use args::*;
use banner::*;
use env_logger::Builder;
use ldap::*;

use modules::*;
use json::checker::*;
use json::maker::make_result;
use json::parser::*;

/// Main of RustHound
#[tokio::main]
async fn main() -> Result<()> {
    // Banner
    print_banner();

    // Get args
    let common_args = extract_args();

    // Build logger
    Builder::new()
        .filter(Some("rusthound"), common_args.verbose)
        .filter_level(log::LevelFilter::Error)
        .init();

    // Get verbose level
    info!("Verbosity level: {:?}", common_args.verbose);

    // Ldap request to get all informations in result
    let result = ldap_search(
        common_args.ldaps,
        &common_args.ip,
        &common_args.port,
        &common_args.domain,
        &common_args.ldapfqdn,
        &common_args.username,
        &common_args.password,
    ).await?;

    // Vector for content all
    let mut vec_users: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_groups: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_computers: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_ous: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_domains: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_gpos: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_fsps: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_containers: Vec<serde_json::value::Value> = Vec::new();
    let mut vec_trusts: Vec<serde_json::value::Value> = Vec::new();
    // Hashmap to link DN to SID
    let mut dn_sid = HashMap::new();
    // Hashmap to link DN to Type
    let mut sid_type = HashMap::new();
    // Hashmap to link FQDN to SID
    let mut fqdn_sid = HashMap::new();
    // Hashmap to link fqdn to an ip address
    let mut fqdn_ip = HashMap::new();

    // Analyze object by object //Get type and parse it to get values
    parse_result_type(
        &common_args.domain,
        result,
        &mut vec_users,
        &mut vec_groups,
        &mut vec_computers,
        &mut vec_ous,
        &mut vec_domains,
        &mut vec_gpos,
        &mut vec_fsps,
        &mut vec_containers,
        &mut vec_trusts,
        &mut dn_sid,
        &mut sid_type,
        &mut fqdn_sid,
        &mut fqdn_ip,
    );

    // Functions to replace and add missing values
    check_all_result(
        &common_args.domain,
        &mut vec_users,
        &mut vec_groups,
        &mut vec_computers,
        &mut vec_ous,
        &mut vec_domains,
        &mut vec_gpos,
        &mut vec_fsps,
        &mut vec_containers,
        &mut vec_trusts,
        &mut dn_sid,
        &mut sid_type,
        &mut fqdn_sid,
        &mut fqdn_ip,
     );

    // Running modules
    run_modules(
        &common_args,
        &mut fqdn_ip,
        &mut vec_computers
    ).await;

    // Add all in json files
    let res = make_result(
        common_args.zip,
        &common_args.path,
        &common_args.domain,
        vec_users,
        vec_groups,
        vec_computers,
        vec_ous,
        vec_domains,
        vec_gpos,
        vec_containers,
    );
    match res {
        Ok(_res) => trace!("Making json/zip files finished!"),
        Err(err) => error!("Error. Reason: {err}")
    }

    // End banner
    print_end_banner();
    Ok(())
}