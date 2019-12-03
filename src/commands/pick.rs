use crate::commands::WholeStreamCommand;
use crate::context::CommandRegistry;
use crate::data::base::select_fields;
use crate::prelude::*;
use nu_errors::ShellError;
use nu_protocol::{Signature, SyntaxShape};
use nu_source::Tagged;

#[derive(Deserialize)]
struct PickArgs {
    rest: Vec<Tagged<String>>,
}

pub struct Pick;

impl WholeStreamCommand for Pick {
    fn name(&self) -> &str {
        "pick"
    }

    fn signature(&self) -> Signature {
        Signature::build("pick").rest(SyntaxShape::Any, "the columns to select from the table")
    }

    fn usage(&self) -> &str {
        "Down-select table to only these columns."
    }

    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        args.process(registry, pick)?.run()
    }
}

fn pick(
    PickArgs { rest: fields }: PickArgs,
    RunnableContext { input, name, .. }: RunnableContext,
) -> Result<OutputStream, ShellError> {
    if fields.len() == 0 {
        return Err(ShellError::labeled_error(
            "Pick requires columns to pick",
            "needs parameter",
            name,
        ));
    }

    let fields: Vec<_> = fields.iter().map(|f| f.item.clone()).collect();

    let objects = input
        .values
        .map(move |value| select_fields(&value, &fields, value.tag.clone()));

    Ok(objects.from_input_stream())
}
