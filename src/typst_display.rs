use std::error::Error;
use std::io::Cursor;
use std::path::Path;
use std::{fs, io};

use base64::Engine;
use comemo::Prehashed;
use typst::diag::{FileError, FileResult};
use typst::eval::Library;
use typst::font::{Font, FontBook};
use typst::geom::{Color, RgbaColor};
use typst::syntax::{Source, SourceId};
use typst::util::Buffer;

use crate::BasicAlgebraicExpr;
use crate::print::print_expr_to_string;

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

pub fn write_image(w: &MyWorld) -> Result<Vec<u8>, Box<dyn Error>> {
    let input = typst::compile(w).unwrap();
    let pixmap = typst::export::render(
        &input.pages[0],
        10.0,
        Color::Rgba(RgbaColor::new(255, 255, 255, 255)),
    );
    let buf = vec![];
    let mut cursor = Cursor::new(buf);
    image::write_buffer_with_format(
        &mut cursor,
        bytemuck::cast_slice(pixmap.pixels()),
        pixmap.width(),
        pixmap.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    Ok(cursor.into_inner())
}

pub const PREAMBLE: &str = r#"

#set page(width: auto, height: auto, margin: 5pt)
#show math.equation: set text(font: "New Computer Modern Math")

"#;

pub fn evcxr_display(exp: &BasicAlgebraicExpr) {
    let str = print_expr_to_string(&exp);
    let world = MyWorld::new(format!("{PREAMBLE} ${str}$")).unwrap();

    let bytes = write_image(&world).unwrap();

    println!("EVCXR_BEGIN_CONTENT image/png\n{}\nEVCXR_END_CONTENT", base64::prelude::BASE64_STANDARD.encode(&bytes));
}