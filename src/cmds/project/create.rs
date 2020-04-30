use anyhow::{Context, Result};

use crate::gitlab::{CreateProjectParams, IfGitLabCreateProject};

pub fn create_project_cmd(
    args: clap::ArgMatches,
    gitlab: impl IfGitLabCreateProject,
) -> Result<()> {
    let mut params = &mut CreateProjectParams::builder();

    // NOTE: args.args is a public, but _undocumented_ field in clap.rs
    // args.args.iter()
    //     .map(
    //         |(k,v)| println!("{} {:?}",*k ,v)
    //         ).count();

    if args.is_present("description") {
        params = params.description(args.value_of("description").unwrap());
        trace!("description: {}", args.value_of("description").unwrap());
    }

    // TODO: add various arg parsing code

    // TODO: Consider changing return value to Result<serde_json::Value> to get raw json.
    let project = gitlab
        .create_project(
            args.value_of("name").unwrap(),
            args.value_of("path"),
            Some(params.build().unwrap()),
        )
        .context("Failed to create project - check for name or path clashes on the server")?;

    println!("Project id: {}", project.id);
    println!("Project URL: {}", project.web_url);
    Ok(())
}

#[cfg(test)]
mod project_create_unit_tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    use anyhow::anyhow;
    use clap::App;
    use gitlab::types::*;
    use serde::de::DeserializeOwned;

    use crate::gitlab::Project;

    use super::*;

    struct GitlabWithMockProject {
        project: Result<Project>,
    }

    impl IfGitLabCreateProject for GitlabWithMockProject {
        fn create_project<N: AsRef<str>, P: AsRef<str>>(
            &self,
            name: N,
            path: Option<P>,
            params: Option<CreateProjectParams>,
        ) -> Result<Project> {
            match &self.project {
                Ok(p) => Ok(p.clone()),
                Err(e) => Err(anyhow!("{}", e)),
            }
        }
    }

    fn load_mock_from_disk<P: AsRef<Path>, T>(path: P) -> T
    where
        T: DeserializeOwned,
    {
        // Open the file in read-only mode with buffer.
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        // Read the JSON contents of the file as an instance of `Project`.
        serde_json::from_reader(reader).unwrap()
    }

    #[test]
    fn test_create_basic_project() {
        // GIVEN:
        let a = App::new("create")
            .arg(
                clap::Arg::with_name("name")
                    .help("Project name")
                    .takes_value(true)
                    .required(true),
            )
            .get_matches_from(vec!["create", "proj_name"]);
        println!("args = {:?}", a);

        let mock_project: Project = load_mock_from_disk("tests/data/project.json");

        let g = GitlabWithMockProject {
            project: Ok(mock_project),
        };

        // WHEN:
        let p = create_project_cmd(a, g);

        // THEN:
    }
}
