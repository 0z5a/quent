// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Non-finite float payloads must survive an ndjson export/import round trip.

use quent_events::{Event, EventPayload, Uuid};
use quent_io_ndjson::{
    NdjsonExporter, NdjsonExporterOptions, NdjsonImporter, NdjsonImporterOptions,
};
use quent_io_types::Exporter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct Sample {
    value: f64,
}

impl EventPayload for Sample {
    const NAME: &'static str = "Sample";
}

#[tokio::test]
async fn non_finite_floats_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let values = [0.25, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
    let mut exporter = NdjsonExporter::try_new::<Sample>(NdjsonExporterOptions {
        dir: dir.path().into(),
    })
    .await
    .unwrap();
    let mut events: Vec<_> = values
        .iter()
        .map(|&value| Event::new(Uuid::nil(), 1, Sample { value }))
        .collect();
    Exporter::<Sample>::drain_events(&mut exporter, &mut events)
        .await
        .unwrap();
    Exporter::<Sample>::shutdown(Box::new(exporter)).await.unwrap();

    let importer = NdjsonImporter::<Sample>::try_new(&NdjsonImporterOptions {
        path: dir.path().join(Sample::NAME),
    })
    .unwrap();
    let imported: Vec<u64> = importer
        .map(|event| event.unwrap().data.value.to_bits())
        .collect();
    let expected: Vec<u64> = values.iter().map(|value| value.to_bits()).collect();
    assert_eq!(imported, expected);
}
