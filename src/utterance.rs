use gcloud_sdk::{
    google::cloud::dialogflow::v2beta1::{
        intents_client::IntentsClient, query_input::Input, sessions_client::SessionsClient,
        DetectIntentRequest, DetectIntentResponse, GetIntentRequest, Intent, IntentView,
        QueryInput, TextInput, UpdateIntentRequest,
    },
    GoogleApi, GoogleAuthMiddleware, GoogleEnvironment,
};
use miette::{IntoDiagnostic, Result};
use nanoid::nanoid;
use once_cell::sync::Lazy;
use std::ops::{Deref, DerefMut};
use tonic::{Request, Response};

const DEFAULT_LANGUAGE_CODE: &str = "en";

pub async fn project_id() -> String {
    GoogleEnvironment::detect_google_project_id().await
    .expect("No Google Project ID detected. Please specify it explicitly using env variable: PROJECT_ID")
}

pub fn session_id() -> String {
    nanoid!(10)
}

pub async fn session_path() -> String {
    format!(
        "projects/{}/agent/sessions/{}",
        project_id().await,
        session_id()
    )
}

#[derive(Clone)]
pub struct DialogflowSession(SessionsClient<GoogleAuthMiddleware>);

impl DialogflowSession {
    pub async fn new() -> Result<Self> {
        Ok(Self(
            GoogleApi::from_function(
                SessionsClient::new,
                "https://dialogflow.googleapis.com",
                None,
            )
            .await
            .into_diagnostic()?
            .get(),
        ))
    }

    pub async fn detect_intent_from_text<S>(
        &mut self,
        text: S,
        language_code: Option<S>,
    ) -> tonic::Result<Response<DetectIntentResponse>>
    where
        S: Into<String> + std::convert::From<&'static str>,
    {
        let language_code: String = language_code.unwrap_or(DEFAULT_LANGUAGE_CODE.into()).into();

        self.0
            .detect_intent(Request::new(DetectIntentRequest {
                session: session_path().await,
                query_input: Some(QueryInput {
                    input: Some(Input::Text(TextInput {
                        text: text.into(),
                        language_code,
                    })),
                }),
                ..Default::default()
            }))
            .await
    }
}

impl Deref for DialogflowSession {
    type Target = SessionsClient<GoogleAuthMiddleware>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DialogflowSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct DialogflowIntent(IntentsClient<GoogleAuthMiddleware>);

impl DialogflowIntent {
    pub async fn new() -> Result<Self> {
        Ok(Self(
            GoogleApi::from_function(
                IntentsClient::new,
                "https://dialogflow.googleapis.com",
                None,
            )
            .await
            .into_diagnostic()?
            .get(),
        ))
    }

    pub async fn get_intent<S>(&mut self, intent_id: S) -> tonic::Result<Response<Intent>>
    where
        S: Into<String> + std::convert::From<&'static str>,
    {
        self.0
            .get_intent(Request::new(GetIntentRequest {
                name: format!(
                    "projects/{}/agent/intents/{}",
                    project_id().await,
                    intent_id.into()
                ),
                language_code: "en".into(),
                intent_view: IntentView::Full.into(),
            }))
            .await
    }

    pub async fn update_intent(&mut self, intent: Intent) -> tonic::Result<Response<Intent>> {
        self.0
            .update_intent(Request::new(UpdateIntentRequest {
                intent: Some(intent),
                intent_view: IntentView::Full.into(),
                ..Default::default()
            }))
            .await
    }
}

impl Deref for DialogflowIntent {
    type Target = IntentsClient<GoogleAuthMiddleware>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DialogflowIntent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// impl Dialogflow {
//     pub async fn new_session() -> Result<Self> {
//         Ok(Self::Session(
//             GoogleApi::from_function(
//                 SessionsClient::new,
//                 "https://dialogflow.googleapis.com",
//                 None,
//             )
//             .await?
//             .get(),
//         ))
//     }

//     pub async fn new_intent() -> Result<Self> {
//         Ok(Self::Intent(
//             GoogleApi::from_function(
//                 IntentsClient::new,
//                 "https://dialogflow.googleapis.com",
//                 None,
//             )
//             .await?
//             .get(),
//         ))
//     }

//     pub async fn project_id() -> String {
//         GoogleEnvironment::detect_google_project_id().await
//             .expect("No Google Project ID detected. Please specify it explicitly using env variable: PROJECT_ID")
//     }

//     pub async fn session_path() -> String {
//         format!(
//             "projects/{}/agent/sessions/{}",
//             Self::project_id().await,
//             DIALOGFLOW_SESSION_ID.to_owned()
//         )
//     }

//     pub async fn detect_intent_from_text<S>(
//         &self,
//         text: S,
//         language_code: Option<S>,
//     ) -> Result<tonic::Response<DetectIntentResponse>, tonic::Status>
//     where
//         S: Into<String> + std::convert::From<&'static str>,
//         Self: Sized,
//     {
//         let language_code: String = language_code.unwrap_or("en-SG".into()).into();

//         .detect_intent(tonic::Request::new(DetectIntentRequest {
//             session: Self::session_path().await,
//             query_input: Some(QueryInput {
//                 input: Some(Input::Text(TextInput {
//                     text: text.into(),
//                     language_code,
//                 })),
//             }),
//             ..Default::default()
//         }))
//         .await
//     }
// }
