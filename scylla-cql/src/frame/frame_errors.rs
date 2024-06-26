use super::TryFromPrimitiveError;
use crate::cql_to_rust::CqlTypeError;
use crate::frame::value::SerializeValuesError;
use crate::types::deserialize::DeserializationError;
use crate::types::serialize::SerializationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("Frame is compressed, but no compression negotiated for connection.")]
    NoCompressionNegotiated,
    #[error("Received frame marked as coming from a client")]
    FrameFromClient,
    #[error("Received frame marked as coming from the server")]
    FrameFromServer,
    #[error("Received a frame from version {0}, but only 4 is supported")]
    VersionNotSupported(u8),
    #[error("Connection was closed before body was read: missing {0} out of {1}")]
    ConnectionClosed(usize, usize),
    #[error("Frame decompression failed.")]
    FrameDecompression,
    #[error("Frame compression failed.")]
    FrameCompression,
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error("Unrecognized opcode{0}")]
    TryFromPrimitiveError(#[from] TryFromPrimitiveError<u8>),
    #[error("Error compressing lz4 data {0}")]
    Lz4CompressError(#[from] lz4_flex::block::CompressError),
    #[error("Error decompressing lz4 data {0}")]
    Lz4DecompressError(#[from] lz4_flex::block::DecompressError),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Set keyspace response deserialization failed: {0}")]
    SetKeyspaceParseError(#[from] SetKeyspaceParseError),
    #[error("Schema change event deserialization failed: {0}")]
    SchemaChangeEventParseError(#[from] SchemaChangeEventParseError),
    #[error("Table spec deserialization failed: {0}")]
    TableSpecParseError(#[from] TableSpecParseError),
    #[error(transparent)]
    TypeParseError(#[from] CqlTypeParseError),
    #[error("Low-level deserialization failed: {0}")]
    LowLevelDeserializationError(#[from] LowLevelDeserializationError),
    #[error("Could not serialize frame: {0}")]
    BadDataToSerialize(String),
    #[error("Could not deserialize frame: {0}")]
    BadIncomingData(String),
    #[error(transparent)]
    DeserializationError(#[from] DeserializationError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerializeValuesError(#[from] SerializeValuesError),
    #[error(transparent)]
    SerializationError(#[from] SerializationError),
    #[error(transparent)]
    CqlTypeError(#[from] CqlTypeError),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SetKeyspaceParseError {
    #[error("Malformed keyspace name: {0}")]
    MalformedKeyspaceName(#[from] LowLevelDeserializationError),
}

/// An error type returned when deserialization of
/// `RESULT::Schema_change` response fails.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum SchemaChangeEventParseError {
    #[error("Malformed schema change type string: {0}")]
    TypeOfChangeParseError(LowLevelDeserializationError),
    #[error("Malformed schema change target string:: {0}")]
    TargetTypeParseError(LowLevelDeserializationError),
    #[error("Malformed name of keyspace affected by schema change: {0}")]
    AffectedKeyspaceParseError(LowLevelDeserializationError),
    #[error("Malformed name of the table affected by schema change: {0}")]
    AffectedTableNameParseError(LowLevelDeserializationError),
    #[error("Malformed name of the target affected by schema change: {0}")]
    AffectedTargetNameParseError(LowLevelDeserializationError),
    #[error(
        "Malformed number of arguments of the function/aggregate affected by schema change: {0}"
    )]
    ArgumentCountParseError(LowLevelDeserializationError),
    #[error("Malformed argument of the function/aggregate affected by schema change: {0}")]
    FunctionArgumentParseError(LowLevelDeserializationError),
    #[error("Unknown target of schema change: {0}")]
    UnknownTargetOfSchemaChange(String),
}

/// An error type returned when deserialization
/// of table specification fails.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum TableSpecParseError {
    #[error("Malformed keyspace name: {0}")]
    MalformedKeyspaceName(LowLevelDeserializationError),
    #[error("Malformed table name: {0}")]
    MalformedTableName(LowLevelDeserializationError),
}

/// An error type returned when deserialization of CQL type name fails.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum CqlTypeParseError {
    #[error("Malformed type id: {0}")]
    TypeIdParseError(LowLevelDeserializationError),
    #[error("Malformed custom type name: {0}")]
    CustomTypeNameParseError(LowLevelDeserializationError),
    #[error("Malformed name of UDT keyspace: {0}")]
    UdtKeyspaceNameParseError(LowLevelDeserializationError),
    #[error("Malformed UDT name: {0}")]
    UdtNameParseError(LowLevelDeserializationError),
    #[error("Malformed UDT fields count: {0}")]
    UdtFieldsCountParseError(LowLevelDeserializationError),
    #[error("Malformed UDT's field name: {0}")]
    UdtFieldNameParseError(LowLevelDeserializationError),
    #[error("Malformed tuple length: {0}")]
    TupleLengthParseError(LowLevelDeserializationError),
    #[error("CQL Type not yet implemented, id: {0}")]
    TypeNotImplemented(u16),
}

/// A low level deserialization error.
///
/// This type of error is returned when deserialization
/// of some primitive value fails.
///
/// Possible error kinds:
/// - generic io error - reading from buffer failed
/// - out of range integer conversion
/// - conversion errors - e.g. slice-to-array or primitive-to-enum
/// - not enough bytes in the buffer to deserialize a value
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum LowLevelDeserializationError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Not enough bytes! expected: {expected}, received: {received}")]
    TooFewBytesReceived { expected: usize, received: usize },
    #[error("Invalid value length: {0}")]
    InvalidValueLength(i32),
    #[error("Unknown consistency: {0}")]
    UnknownConsistency(#[from] TryFromPrimitiveError<u16>),
    #[error("Invalid inet bytes length: {0}. Accepted lengths are 4 and 16 bytes.")]
    InvalidInetLength(u8),
    #[error("UTF8 deserialization failed: {0}")]
    UTF8DeserializationError(#[from] std::str::Utf8Error),
}
