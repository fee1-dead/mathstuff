use std::error::Error;
use std::io::{Write, Seek};
use std::io;
use std::path::Path;

use comemo::Prehashed;
use palette::rgb::Rgba;
use typst::diag::{FileError, FileResult};
use typst::eval::{Library, Tracer, Bytes};
use typst::font::{Font, FontBook};
use typst::geom::Color;
use typst::syntax::{Source, FileId, VirtualPath};

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
        let data = include_bytes!("../font/NewCMMath-Regular.otf");
        let fonts =  Font::iter(data.as_slice().into()).collect::<Vec<_>>();
        let book = FontBook::from_fonts(&fonts);
        Ok(Self {
            library: Prehashed::new(typst_library::build()),
            fonts,
            book: Prehashed::new(book),
            main: Source::new(FileId::new(None, VirtualPath::new("/input")), src),
        })
    }

    pub fn set_source(&mut self, src: String) {
        self.main = Source::new(FileId::new(None, VirtualPath::new("/input")), src);
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


    fn file(&self, path: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(path.vpath().as_rooted_path().into()))
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        assert_eq!(Path::new("/input"), id.vpath().as_rooted_path());
        Ok(self.main.clone())
    }

    fn main(&self) -> Source {
        self.main.clone()
    }

    fn today(&self, _offset: Option<i64>) -> Option<typst::eval::Datetime>  {
        None
    }
}

pub fn write_image(w: &MyWorld, x: &mut (impl Write + Seek)) -> Result<(), Box<dyn Error>> {
    let mut tracer = Tracer::new();
    let input = typst::compile(w, &mut tracer).unwrap();
    let pixmap = typst::export::render(
        &input.pages[0],
        10.0,
        Color::Rgba(Rgba::new(1., 1., 1., 1.)),
    );
    image::write_buffer_with_format(
        x,
        bytemuck::cast_slice(pixmap.pixels()),
        pixmap.width(),
        pixmap.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    Ok(())
}

pub const PREAMBLE: &str = r#"

#set page(width: auto, height: auto, margin: 5pt)
#show math.equation: set text(font: "New Computer Modern Math")

"#;

/* 
pub fn evcxr_display(exp: &BasicAlgebraicExpr) {
    let str = print_expr_to_string(&exp);
    let world = MyWorld::new(format!("{PREAMBLE} ${str}$")).unwrap();

    let mut bytes = Cursor::new(Vec::new());
    write_image(&world, &mut bytes);

    let bytes = bytes.into_inner();
    println!("EVCXR_BEGIN_CONTENT image/png\n{}\nEVCXR_END_CONTENT", base64::prelude::BASE64_STANDARD.encode(&bytes));
}
*/