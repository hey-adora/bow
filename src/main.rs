use bow::editor::{self, App};



fn main() -> anyhow::Result<()> {
    let mut editor = editor::Editor::<editor::frontend::Terminal>::new()?;

    editor.run()?;

    Ok(())
}

