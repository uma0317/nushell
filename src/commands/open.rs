use crate::commands::UnevaluatedCallInfo;
use crate::data::value;
use crate::prelude::*;
use nu_errors::ShellError;
use nu_protocol::{CallInfo, ReturnSuccess, Signature, SyntaxShape, UntaggedValue, Value};
use nu_source::{AnchorLocation, Span};
use std::path::{Path, PathBuf};

pub struct Open;

impl PerItemCommand for Open {
    fn name(&self) -> &str {
        "open"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "path",
                SyntaxShape::Path,
                "the file path to load values from",
            )
            .switch("raw", "load content as a string insead of a table")
    }

    fn usage(&self) -> &str {
        "Load a file into a cell, convert to table if possible (avoid by appending '--raw')"
    }

    fn run(
        &self,
        call_info: &CallInfo,
        registry: &CommandRegistry,
        raw_args: &RawCommandArgs,
        _input: Value,
    ) -> Result<OutputStream, ShellError> {
        run(call_info, registry, raw_args)
    }
}

fn run(
    call_info: &CallInfo,
    registry: &CommandRegistry,
    raw_args: &RawCommandArgs,
) -> Result<OutputStream, ShellError> {
    let shell_manager = &raw_args.shell_manager;
    let cwd = PathBuf::from(shell_manager.path());
    let full_path = PathBuf::from(cwd);

    let path = call_info.args.nth(0).ok_or_else(|| {
        ShellError::labeled_error(
            "No file or directory specified",
            "for command",
            &call_info.name_tag,
        )
    })?;

    let path_buf = path.as_path()?;
    let path_str = path_buf.display().to_string();
    let path_span = path.tag.span;
    let has_raw = call_info.args.has("raw");
    let registry = registry.clone();
    let raw_args = raw_args.clone();

    let stream = async_stream! {

        let result = fetch(&full_path, &path_str, path_span).await;

        if let Err(e) = result {
            yield Err(e);
            return;
        }
        let (file_extension, contents, contents_tag) = result.unwrap();

        let file_extension = if has_raw {
            None
        } else {
            // If the extension could not be determined via mimetype, try to use the path
            // extension. Some file types do not declare their mimetypes (such as bson files).
            file_extension.or(path_str.split('.').last().map(String::from))
        };

        let tagged_contents = contents.into_value(&contents_tag);

        if let Some(extension) = file_extension {
            let command_name = format!("from-{}", extension);
            if let Some(converter) = registry.get_command(&command_name) {
                let new_args = RawCommandArgs {
                    host: raw_args.host,
                    ctrl_c: raw_args.ctrl_c,
                    shell_manager: raw_args.shell_manager,
                    call_info: UnevaluatedCallInfo {
                        args: nu_parser::hir::Call {
                            head: raw_args.call_info.args.head,
                            positional: None,
                            named: None,
                            span: Span::unknown()
                        },
                        source: raw_args.call_info.source,
                        name_tag: raw_args.call_info.name_tag,
                    }
                };
                let mut result = converter.run(new_args.with_input(vec![tagged_contents]), &registry);
                let result_vec: Vec<Result<ReturnSuccess, ShellError>> = result.drain_vec().await;
                for res in result_vec {
                    match res {
                        Ok(ReturnSuccess::Value(Value { value: UntaggedValue::Table(list), ..})) => {
                            for l in list {
                                yield Ok(ReturnSuccess::Value(l));
                            }
                        }
                        Ok(ReturnSuccess::Value(Value { value, .. })) => {
                            yield Ok(ReturnSuccess::Value(Value { value, tag: contents_tag.clone() }));
                        }
                        x => yield x,
                    }
                }
            } else {
                yield ReturnSuccess::value(tagged_contents);
            }
        } else {
            yield ReturnSuccess::value(tagged_contents);
        }
    };

    Ok(stream.to_output_stream())
}

pub async fn fetch(
    cwd: &PathBuf,
    location: &str,
    span: Span,
) -> Result<(Option<String>, UntaggedValue, Tag), ShellError> {
    let mut cwd = cwd.clone();

    cwd.push(Path::new(location));
    if let Ok(cwd) = dunce::canonicalize(cwd) {
        match std::fs::read(&cwd) {
            Ok(bytes) => match std::str::from_utf8(&bytes) {
                Ok(s) => Ok((
                    cwd.extension()
                        .map(|name| name.to_string_lossy().to_string()),
                    value::string(s),
                    Tag {
                        span,
                        anchor: Some(AnchorLocation::File(cwd.to_string_lossy().to_string())),
                    },
                )),
                Err(_) => {
                    //Non utf8 data.
                    match (bytes.get(0), bytes.get(1)) {
                        (Some(x), Some(y)) if *x == 0xff && *y == 0xfe => {
                            // Possibly UTF-16 little endian
                            let utf16 = read_le_u16(&bytes[2..]);

                            if let Some(utf16) = utf16 {
                                match std::string::String::from_utf16(&utf16) {
                                    Ok(s) => Ok((
                                        cwd.extension()
                                            .map(|name| name.to_string_lossy().to_string()),
                                        value::string(s),
                                        Tag {
                                            span,
                                            anchor: Some(AnchorLocation::File(
                                                cwd.to_string_lossy().to_string(),
                                            )),
                                        },
                                    )),
                                    Err(_) => Ok((
                                        None,
                                        value::binary(bytes),
                                        Tag {
                                            span,
                                            anchor: Some(AnchorLocation::File(
                                                cwd.to_string_lossy().to_string(),
                                            )),
                                        },
                                    )),
                                }
                            } else {
                                Ok((
                                    None,
                                    value::binary(bytes),
                                    Tag {
                                        span,
                                        anchor: Some(AnchorLocation::File(
                                            cwd.to_string_lossy().to_string(),
                                        )),
                                    },
                                ))
                            }
                        }
                        (Some(x), Some(y)) if *x == 0xfe && *y == 0xff => {
                            // Possibly UTF-16 big endian
                            let utf16 = read_be_u16(&bytes[2..]);

                            if let Some(utf16) = utf16 {
                                match std::string::String::from_utf16(&utf16) {
                                    Ok(s) => Ok((
                                        cwd.extension()
                                            .map(|name| name.to_string_lossy().to_string()),
                                        value::string(s),
                                        Tag {
                                            span,
                                            anchor: Some(AnchorLocation::File(
                                                cwd.to_string_lossy().to_string(),
                                            )),
                                        },
                                    )),
                                    Err(_) => Ok((
                                        None,
                                        value::binary(bytes),
                                        Tag {
                                            span,
                                            anchor: Some(AnchorLocation::File(
                                                cwd.to_string_lossy().to_string(),
                                            )),
                                        },
                                    )),
                                }
                            } else {
                                Ok((
                                    None,
                                    value::binary(bytes),
                                    Tag {
                                        span,
                                        anchor: Some(AnchorLocation::File(
                                            cwd.to_string_lossy().to_string(),
                                        )),
                                    },
                                ))
                            }
                        }
                        _ => Ok((
                            None,
                            value::binary(bytes),
                            Tag {
                                span,
                                anchor: Some(AnchorLocation::File(
                                    cwd.to_string_lossy().to_string(),
                                )),
                            },
                        )),
                    }
                }
            },
            Err(_) => {
                return Err(ShellError::labeled_error(
                    "File could not be opened",
                    "file not found",
                    span,
                ));
            }
        }
    } else {
        return Err(ShellError::labeled_error(
            "File could not be opened",
            "file not found",
            span,
        ));
    }
}

fn read_le_u16(input: &[u8]) -> Option<Vec<u16>> {
    if input.len() % 2 != 0 || input.len() < 2 {
        None
    } else {
        let mut result = vec![];
        let mut pos = 0;
        while pos < input.len() {
            result.push(u16::from_le_bytes([input[pos], input[pos + 1]]));
            pos += 2;
        }

        Some(result)
    }
}

fn read_be_u16(input: &[u8]) -> Option<Vec<u16>> {
    if input.len() % 2 != 0 || input.len() < 2 {
        None
    } else {
        let mut result = vec![];
        let mut pos = 0;
        while pos < input.len() {
            result.push(u16::from_be_bytes([input[pos], input[pos + 1]]));
            pos += 2;
        }

        Some(result)
    }
}
