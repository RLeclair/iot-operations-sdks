// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use serde::Deserialize;

use crate::metl::test_case_catch::TestCaseCatch;
use crate::metl::test_case_published_message::TestCasePublishedMessage;
use crate::metl::test_case_received_telemetry::TestCaseReceivedTelemetry;

#[derive(Clone, Deserialize, Debug)]
#[allow(dead_code)]
pub struct TestCaseEpilogue {
    #[serde(rename = "subscribed-topics")]
    #[serde(default)]
    pub subscribed_topics: Vec<String>,

    #[serde(rename = "publication-count")]
    pub publication_count: Option<i32>,

    #[serde(rename = "published-messages")]
    #[serde(default)]
    pub published_messages: Vec<TestCasePublishedMessage>,

    #[serde(rename = "acknowledgement-count")]
    pub acknowledgement_count: Option<i32>,

    #[serde(rename = "received-telemetries")]
    #[serde(default)]
    pub received_telemetries: Vec<TestCaseReceivedTelemetry>,

    #[serde(rename = "execution-count")]
    pub execution_count: Option<i32>,

    #[serde(rename = "execution-counts")]
    #[serde(default)]
    pub execution_counts: HashMap<usize, i32>,

    #[serde(rename = "telemetry-count")]
    pub telemetry_count: Option<i32>,

    #[serde(rename = "telemetry-counts")]
    #[serde(default)]
    pub telemetry_counts: HashMap<usize, i32>,

    #[serde(rename = "catch")]
    pub catch: Option<TestCaseCatch>,
}
