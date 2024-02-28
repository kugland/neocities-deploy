use crate::{params::Params, trees};
use anyhow::Result;
use bytesize::ByteSize;

/// List files on the site(s).
pub fn list(params: &Params) -> Result<()> {
    for (name, site) in params.sites()? {
        println!("Listing site {}", name);
        let client = site.build_client()?;
        let list = client.list().or_else(|e| {
            if params.ignore_errors {
                log::error!("{}", e);
                Ok(vec![])
            } else {
                Err(e)
            }
        })?;
        let remote = trees::remote_tree(&list);
        for entry in remote {
            let (size, path) = if let Some(info) = entry.info {
                (format!("{}", ByteSize(info.size)), entry.path)
            } else {
                ("".to_owned(), format!("{}/", entry.path))
            };
            println!("{:>10}  {}", size, path);
        }
    }
    Ok(())
}
