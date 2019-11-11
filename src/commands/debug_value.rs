use crate::commands::WholeStreamCommand;
use crate::data::{Primitive, Value};
use crate::prelude::*;
use crate::RawPathMember;
use futures_util::pin_mut;

pub struct DebugValue;

#[derive(Deserialize)]
pub struct DebugArgs {}

impl WholeStreamCommand for DebugValue {
    fn name(&self) -> &str {
        "debug"
    }

    fn signature(&self) -> Signature {
        Signature::build("debug")
    }

    fn usage(&self) -> &str {
        "Print the Rust debug representation of the values"
    }

    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        args.process(registry, debug_value)?.run()
    }
}

fn debug_value(
    args: DebugArgs,
    RunnableContext { input, .. }: RunnableContext,
) -> Result<OutputStream, ShellError> {
    let stream = async_stream! {
        let values = input.values;
        pin_mut!(values);
        while let Some(row) = values.next().await {
            yield ReturnSuccess::debug_value(row.clone())
        }
    };

    let stream: BoxStream<'static, ReturnValue> = stream.boxed();

    Ok(stream.to_output_stream())
}
