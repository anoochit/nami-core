use termimad::{MadSkin, mad_print_inline};

pub async fn download_skills(skin: &MadSkin) -> anyhow::Result<()> {
    let nami_dir = crate::utils::get_nami_dir();
    let skills_dir = nami_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    if inquire::Confirm::new("Do you want to download/update official Nami skills from GitHub?")
        .with_default(true)
        .prompt()?
    {
        mad_print_inline!(skin, "\nDownloading skills from github.com/anoochit/nami-skills...\n");
        
        let client = crate::utils::get_http_client().clone();
        let response = client
            .get("https://github.com/anoochit/nami-skills/archive/refs/heads/master.zip")
            .header("User-Agent", "nami-cli")
            .send()
            .await?;
            
        if !response.status().is_success() {
            anyhow::bail!("Failed to download skills: HTTP {}", response.status());
        }
        
        let bytes = response.bytes().await?;
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path,
                None => continue,
            };
            
            // Strip the top level directory (e.g. `nami-skills-main/`)
            let mut components = outpath.components();
            components.next(); // Skip root folder
            let sub_path = components.as_path();
            if sub_path.as_os_str().is_empty() {
                continue;
            }
            
            let dest_path = skills_dir.join(sub_path);
            
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(p) = dest_path.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&dest_path)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        
        mad_print_inline!(skin, "\n**Success!** Skills successfully downloaded and extracted into `~/.nami/skills/`.\n");
    }
    Ok(())
}
