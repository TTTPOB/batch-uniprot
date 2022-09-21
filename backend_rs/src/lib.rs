pub mod uniprot_proxy {
    use super::uniprot::idmapping;
    use serde_derive::Deserialize;
    use serde_derive::Serialize;
    use std::error::Error;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Query {
        pub tax_id: i64,
        pub ids: Vec<String>,
        pub from: Option<String>,
        pub to: Option<String>,
    }

    pub async fn make_query(q: &Query) -> Result<String, Box<dyn Error>> {
        let query = idmapping::Query::from(q);
        let query_result = idmapping::idmapping_wrapper(&query).await.unwrap();
        Ok(query_result)
    }
}

pub mod uniprot {
    pub mod idmapping {
        use reqwest;
        use serde_derive::Deserialize;
        use serde_derive::Serialize;

        const BASE_URL: &str = "https://rest.uniprot.org/idmapping";

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Query {
            from: Option<String>,
            to: Option<String>,
            ids: String,
            tax_id: i64,
        }

        impl Query {
            fn default_from_to(&mut self) {
                if let None = self.from {
                    self.from = Some("Gene_Name".to_string());
                }

                if let None = self.to {
                    self.to = Some("UniProtKB-Swiss-Prot".to_string());
                }
            }
        }

        impl From<&super::super::uniprot_proxy::Query> for Query {
            fn from(query: &super::super::uniprot_proxy::Query) -> Self {
                let mut q = Query {
                    from: query.from.clone(),
                    to: query.to.clone(),
                    ids: query.ids.join(","),
                    tax_id: query.tax_id,
                };
                q.default_from_to();
                q
            }
        }

        // todo: job_status value as enum
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct JobStatus {
            pub job_status: String,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct JobId {
            pub job_id: String,
        }

        async fn submit_job(q: Query) -> Result<JobId, reqwest::Error> {
            let client = reqwest::Client::new();
            let res: JobId = client
                .post(BASE_URL.to_string() + "/run")
                .form(&q)
                .send()
                .await?
                .json()
                .await?;
            Ok(res)
        }

        async fn get_job_result(job_id: &JobId) -> Result<JobStatus, reqwest::Error> {
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let res: JobStatus = client
                .get(BASE_URL.to_string() + "/status/" + job_id.job_id.as_str())
                .send()
                .await?
                .json()
                .await?;
            Ok(res)
        }

        async fn get_query_result(job_id: &JobId) -> Result<String, reqwest::Error> {
            let client = reqwest::Client::new();
            let res: String = client
                .get(
                    BASE_URL.to_string()
                        + "/uniprotkb/results/"
                        + job_id.job_id.as_str()
                        + "?compressed=false&format=json",
                )
                .send()
                .await?
                .text()
                .await?;
            Ok(res)
        }

        async fn wait_till_job_done_and_get_result(
            job_id: &JobId,
            timeout: i32,
        ) -> Result<String, reqwest::Error> {
            let mut job_status = get_job_result(job_id).await?;
            for _ in 0..timeout {
                if job_status.job_status == "FINISHED" {
                    return Ok(get_query_result(job_id).await.unwrap());
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                job_status = get_job_result(job_id).await?;
            }
            panic!("Job timeout");
        }

        pub async fn idmapping_wrapper(q: &Query) -> Result<String, Box<dyn std::error::Error>> {
            let job_id = submit_job(q.clone()).await?;
            let query_result = wait_till_job_done_and_get_result(&job_id, 10).await?;
            Ok(query_result)
        }

        mod tests {
            use super::*;
            use serde_json;
            use std::fs;
            use tokio_test;

            fn setup_query() -> Query {
                let q_json = fs::read_to_string("fixtures/query.json").unwrap();
                let q: super::super::super::uniprot_proxy::Query =
                    serde_json::from_str(q_json.as_str()).unwrap();
                let real_q = Query::from(&q);
                real_q
            }

            #[tokio::test]
            async fn test_submit_job() {
                let q = setup_query();
                let job_id = submit_job(q).await.unwrap();
                println!("{}", job_id.job_id);
            }

            #[tokio::test]
            async fn test_job_status() {
                let q = setup_query();
                let job_id = submit_job(q).await.unwrap();
                let job_status = get_job_result(&job_id).await.unwrap();
                println!("job status: {}", job_status.job_status);
            }

            #[tokio::test]
            async fn test_whole_query() {
                let q = setup_query();
                let job_id = submit_job(q).await.unwrap();
                let query_result = wait_till_job_done_and_get_result(&job_id, 30)
                    .await
                    .unwrap();
                println!("{}", query_result);
            }
        }
    }
}
