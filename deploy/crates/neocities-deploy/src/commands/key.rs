use crate::params::Params;
use anyhow::Result;
use neocities_client::Auth;

/// Replace credentials with API keys in the config file.
pub fn key(params: &Params) -> Result<()> {
    let sites: Vec<_> = (params.sites()?)
        .into_iter()
        .filter(|(_, site)| matches!(site.auth, Auth::Credentials(_, _)))
        .collect();

    if sites.is_empty() {
        eprintln!("No sites to get API keys for.");
        return Ok(());
    }

    let mut config = params.config()?;
    for (name, site) in sites {
        if matches!(site.auth, Auth::ApiKey(_)) {
            continue;
        }
        println!("Getting API key for site {}", name);
        let client = site.build_client()?;
        let key = match client.key() {
            Ok(key) => Ok(key),
            Err(e) => {
                if !params.ignore_errors {
                    Err(e)
                } else {
                    log::error!("{}", e);
                    continue;
                }
            }
        }?;
        config.sites.get_mut(&name).unwrap().auth = Auth::ApiKey(key);
    }
    config.save(params.config_file())?;
    Ok(())
}
