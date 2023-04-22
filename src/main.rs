use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::{fs, io};

use comemo::Prehashed;
use mathstuff::parse::parse_into_expression;
use mathstuff::print::print_expr_to_string;
use typst::diag::{FileError, FileResult};
use typst::eval::Library;
use typst::font::{Font, FontBook};
use typst::geom::{Color, RgbaColor};
use typst::syntax::{Source, SourceId};
use typst::util::Buffer;

pub struct MyWorld {
    library: Prehashed<Library>,
    fonts: Vec<Font>,
    book: Prehashed<FontBook>,
    main: Source,
}

impl MyWorld {
    pub fn new(src: String) -> io::Result<Self> {
        //let fonts_list = Command::new("fc-list").arg("-f").arg("%{file}\n").output()?;
        //let s = String::from_utf8_lossy(&fonts_list.stdout);
        let s = "./font/NewCMMath-Regular.otf";
        let fonts = s
            .lines()
            .flat_map(|x| fs::read(x).map(|data| Font::iter(data.into())).ok())
            .flatten()
            .collect::<Vec<_>>();

        dbg!(&fonts);

        let book = FontBook::from_fonts(fonts.iter());
        Ok(Self {
            library: Prehashed::new(typst_library::build()),
            fonts,
            book: Prehashed::new(book),
            main: Source::new(SourceId::from_u16(0), Path::new("input"), src),
        })
    }

    pub fn set_source(&mut self, src: String) {
        self.main = Source::new(SourceId::from_u16(0), Path::new("input"), src);
    }
}

impl typst::World for MyWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.get(id).cloned()
    }

    fn book(&self) -> &Prehashed<typst::font::FontBook> {
        &self.book
    }

    fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        Err(FileError::NotFound(path.into()))
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        Err(FileError::NotFound(path.into()))
    }

    fn source(&self, id: SourceId) -> &Source {
        assert_eq!(0, id.into_u16());
        &self.main
    }

    fn main(&self) -> &Source {
        &self.main
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let eq = parse_into_expression(&fs::read_to_string("./test.txt")?).unwrap();
    let str = print_expr_to_string(&eq);
    let world = MyWorld::new(format!(
        r#"
        #set page(width: auto, height: auto, margin: 5pt)
        #show math.equation: set text(font: "New Computer Modern Math")
        ${str}$
    "#
    ))?;
    println!("debug uwu");
    let input = typst::compile(&world).unwrap();
    let pixmap = typst::export::render(
        &input.pages[0],
        10.0,
        Color::Rgba(RgbaColor::new(255, 255, 255, 255)),
    );
    image::write_buffer_with_format(
        &mut File::create("./out.png")?,
        bytemuck::cast_slice(pixmap.pixels()),
        pixmap.width(),
        pixmap.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    Ok(())
}
