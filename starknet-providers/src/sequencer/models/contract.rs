use std::{fmt::Formatter, io::Write};

use flate2::{write::GzEncoder, Compression};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::serde_as;
use starknet_core::{
    serde::{byte_array::base64::serialize as base64_ser, unsigned_field_element::UfeHex},
    types::{
        contract::{
            legacy::{LegacyContractClass, RawLegacyAbiEntry, RawLegacyEntryPoints},
            CompressProgramError,
        },
        EntryPointsByType, Felt, FlattenedSierraClass,
    },
};

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum DeployedClass {
    SierraClass(FlattenedSierraClass),
    LegacyClass(LegacyContractClass),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
pub struct CompressedSierraClass {
    #[serde(serialize_with = "base64_ser")]
    pub sierra_program: Vec<u8>,
    pub contract_class_version: String,
    pub entry_points_by_type: EntryPointsByType,
    pub abi: String,
}

/// This type exists because of an `offset` issue. Without this type declaration of pre 0.11.0
/// contracts against the sequencer gateway won't function properly.
#[derive(Debug, Serialize, Clone)]
pub struct CompressedLegacyContractClass {
    #[serde(serialize_with = "base64_ser")]
    pub program: Vec<u8>,
    pub entry_points_by_type: RawLegacyEntryPoints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<Vec<RawLegacyAbiEntry>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DecompressProgramError {
    #[error("json deserialization error: {0}")]
    Json(serde_json::Error),
    #[error("decompression io error: {0}")]
    Io(std::io::Error),
}

impl DeployedClass {
    // We need to manually implement this because `raw_value` doesn't work with `untagged`:
    // https://github.com/serde-rs/serde/issues/1183
    // Preventing us to impl `Deserialize` on DeployedClass in a satisfying way.
    pub fn deserialize_from_json_deployed_class(json: &[u8]) -> Result<Self, serde_json::Error> {
        // Those are the fields of both `FlattenedSierraClass` and `LegacyClass` mixed together.
        // We don't deserialize their content, we just check whether or not those fields are present.
        #[derive(Deserialize)]
        struct BulkDeployedClassFields<'a> {
            // Common fields
            #[serde(borrow)]
            pub entry_points_by_type: &'a serde_json::value::RawValue,
            #[serde(borrow)]
            pub abi: &'a serde_json::value::RawValue,
            // Sierra contract specific fields
            #[serde(borrow)]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub sierra_program: Option<&'a serde_json::value::RawValue>,
            #[serde(borrow)]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub contract_class_version: Option<&'a serde_json::value::RawValue>,
            // Cairo countract specific field
            #[serde(borrow)]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub program: Option<&'a serde_json::value::RawValue>,
        }

        let buld_fields: BulkDeployedClassFields = serde_json::from_slice(json).unwrap();

        let deployed_class = match buld_fields.program {
            Some(program) => DeployedClass::LegacyClass(LegacyContractClass {
                abi: serde_json::from_str(buld_fields.abi.get())?,
                entry_points_by_type: serde_json::from_str(buld_fields.entry_points_by_type.get())?,
                program: serde_json::from_str(program.get())?,
            }),
            None => DeployedClass::SierraClass(FlattenedSierraClass {
                sierra_program: serde_json::from_str(
                    buld_fields
                        .sierra_program
                        .ok_or(serde_json::Error::missing_field("sierra_program"))?
                        .get(),
                )?,
                contract_class_version: serde_json::from_str(
                    buld_fields
                        .contract_class_version
                        .ok_or(serde_json::Error::missing_field("contract_class_version"))?
                        .get(),
                )?,
                entry_points_by_type: serde_json::from_str(buld_fields.entry_points_by_type.get())?,
                abi: serde_json::from_str(buld_fields.abi.get())?,
            }),
        };

        Ok(deployed_class)
    }
}

impl CompressedSierraClass {
    pub fn from_flattened(
        flattened_class: &FlattenedSierraClass,
    ) -> Result<Self, DecompressProgramError> {
        #[serde_as]
        #[derive(Serialize)]
        struct SierraProgram<'a>(#[serde_as(as = "Vec<UfeHex>")] &'a Vec<Felt>);

        let program_json = serde_json::to_string(&SierraProgram(&flattened_class.sierra_program))
            .map_err(DecompressProgramError::Json)?;

        // Use best compression level to optimize for payload size
        let mut gzip_encoder = GzEncoder::new(Vec::new(), Compression::best());
        gzip_encoder
            .write_all(program_json.as_bytes())
            .map_err(DecompressProgramError::Io)?;

        let compressed_program = gzip_encoder.finish().map_err(DecompressProgramError::Io)?;

        Ok(CompressedSierraClass {
            sierra_program: compressed_program,
            contract_class_version: flattened_class.contract_class_version.clone(),
            entry_points_by_type: flattened_class.entry_points_by_type.clone(),
            abi: flattened_class.abi.clone(),
        })
    }
}
