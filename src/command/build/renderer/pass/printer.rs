use super::Pass;

#[derive(Default)]
pub struct PrinterPass {}

impl Pass for PrinterPass {
    fn run<'a>(
        &self,
        events: impl Iterator<Item = pulldown_cmark::Event<'a>>,
    ) -> impl Iterator<Item = pulldown_cmark::Event<'a>> {
        events.map(|e| {
            println!("{:?}", e);
            e
        })
    }
}
