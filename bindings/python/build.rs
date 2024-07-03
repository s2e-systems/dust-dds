use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

use syn::{visit::Visit, Attribute, Ident, ImplItemFn};

fn write_type(pyi_file: &mut fs::File, type_path: &syn::Type) {
    match type_path {
        syn::Type::Path(p) => {
            // If the TypePath has an ident it directly represents a type, otherwise
            // it is a generic
            if let Some(i) = p.path.get_ident() {
                let type_name = match i.to_string().as_str() {
                    "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" => {
                        "int".to_string()
                    }
                    "String" => "str".to_string(),
                    _ => i.to_string(),
                };
                write!(pyi_file, "{}", type_name).unwrap();
            } else {
                let type_name = p.path.segments[0].ident.to_string();
                // write!(self.pyi_file, ": {:?}", p.path.segments[0]).unwrap();
                match type_name.as_str() {
                    "Option" => {
                        match &p.path.segments[0].arguments {
                            syn::PathArguments::AngleBracketed(g) => match &g.args[0] {
                                syn::GenericArgument::Type(t) => write_type(pyi_file, t),
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        };
                        write!(pyi_file, " | None ").unwrap();
                    }
                    "Vec" => {
                        write!(pyi_file, "list[").unwrap();
                        match &p.path.segments[0].arguments {
                            syn::PathArguments::AngleBracketed(g) => match &g.args[0] {
                                syn::GenericArgument::Type(t) => write_type(pyi_file, t),
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        }
                        write!(pyi_file, "]").unwrap();
                    }
                    "Py" => write!(pyi_file, "Any").unwrap(),
                    "PyResult" => match &p.path.segments[0].arguments {
                        syn::PathArguments::AngleBracketed(g) => match &g.args[0] {
                            syn::GenericArgument::Type(t) => write_type(pyi_file, t),
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    },
                    _ => {
                        unimplemented!();
                    }
                }
            }
        }
        syn::Type::Reference(r) => write_type(pyi_file, r.elem.as_ref()),
        syn::Type::Tuple(t) => {
            if t.elems.is_empty() {
                write!(pyi_file, "None").unwrap();
            } else {
                todo!()
            }
        }
        syn::Type::Array(a) => {
            write!(pyi_file, "list[").unwrap();
            write_type(pyi_file, a.elem.as_ref());
            write!(pyi_file, "]").unwrap();
        }
        syn::Type::Slice(s) => {
            write!(pyi_file, "list[").unwrap();
            write_type(pyi_file, s.elem.as_ref());
            write!(pyi_file, "]").unwrap();
        }
        _ => {
            write!(pyi_file, "Unimplemented {:?}", type_path).unwrap();
            todo!();
        }
    }
}

fn write_docs(pyi_file: &mut fs::File, attrs: &Vec<Attribute>) {
    for attr in attrs.iter() {
        match &attr.meta {
            syn::Meta::NameValue(n) => {
                if let Some(path_ident) = n.path.get_ident() {
                    if path_ident == "doc" {
                        match &n.value {
                            syn::Expr::Lit(l) => match &l.lit {
                                syn::Lit::Str(s) => {
                                    let doc_string = s.token().to_string();
                                    writeln!(
                                        pyi_file,
                                        "{}",
                                        &doc_string[1..doc_string.len() - 1] // Do not print the initial and final ""
                                    )
                                    .unwrap();
                                }
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

fn write_default_value(pyi_file: &mut fs::File, field_ident: &Ident, attrs: &Vec<Attribute>) {
    for attr in attrs.iter() {
        match &attr.meta {
            syn::Meta::List(l) => {
                if l.path.segments[0].ident == "pyo3" {
                    if let Ok(syn::Meta::NameValue(n)) = l.parse_args::<syn::Meta>() {
                        if n.path.segments[0].ident == "signature" {
                            match &n.value {
                                syn::Expr::Tuple(t) => {
                                    for elem in t.elems.iter() {
                                        match elem {
                                            syn::Expr::Assign(a) => match a.left.as_ref() {
                                                syn::Expr::Path(p) => {
                                                    if p.path.get_ident().unwrap() == field_ident {
                                                        match a.right.as_ref() {
                                                            syn::Expr::Call(right_call) => {
                                                                match right_call.func.as_ref() {
                                                                    syn::Expr::Path(call_path) => {
                                                                        if call_path.path.segments
                                                                            [1]
                                                                        .ident
                                                                            == "default"
                                                                        {
                                                                            write!(
                                                                                pyi_file,
                                                                                " = ...",
                                                                            )
                                                                            .unwrap()
                                                                        } else if call_path
                                                                            .path
                                                                            .segments[0]
                                                                            .ident
                                                                            == "Vec"
                                                                            && call_path
                                                                                .path
                                                                                .segments[1]
                                                                                .ident
                                                                                == "new"
                                                                        {
                                                                            write!(
                                                                                pyi_file,
                                                                                " = []"
                                                                            )
                                                                            .unwrap()
                                                                        } else {
                                                                            write!(pyi_file, "Assign call path {:?}", call_path.path).unwrap();
                                                                            todo!()
                                                                        }
                                                                    }
                                                                    _ => unreachable!(),
                                                                }
                                                            }
                                                            syn::Expr::MethodCall(method_call) => {
                                                                match method_call.receiver.as_ref()
                                                                {
                                                                    syn::Expr::Path(
                                                                        receiver_path,
                                                                    ) => write!(
                                                                        pyi_file,
                                                                        " = {}",
                                                                        receiver_path.path.segments
                                                                            [0]
                                                                        .ident
                                                                    )
                                                                    .unwrap(),
                                                                    _ => unreachable!(),
                                                                }
                                                            }
                                                            syn::Expr::Path(right_path) => {
                                                                write!(
                                                                    pyi_file,
                                                                    " = {}",
                                                                    right_path
                                                                        .path
                                                                        .get_ident()
                                                                        .unwrap()
                                                                )
                                                                .unwrap();
                                                                break;
                                                            }
                                                            _ => unreachable!(),
                                                        }
                                                    }
                                                }
                                                _ => unreachable!(),
                                            },
                                            syn::Expr::Path(p) => {
                                                if p.path.get_ident().unwrap() == field_ident {
                                                    break;
                                                }
                                            } // No default value when mentioned only by name
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }

                //
            }
            _ => (),
        }
    }
}

struct PyiImplVisitor<'ast> {
    pyi_file: &'ast mut File,
    class_name: String,
    is_empty: bool,
    dust_dds_ast_file: &'ast syn::File,
}

impl<'ast> PyiImplVisitor<'ast> {
    fn write_fn_item(&mut self, fn_item: &ImplItemFn) {
        fn is_constructor(fn_item: &ImplItemFn) -> bool {
            fn_item
                .attrs
                .iter()
                .filter_map(|a| a.meta.path().get_ident())
                .any(|i| *i == "new")
        }

        let fn_name = fn_item.sig.ident.to_string();
        // From conversions and as_ref are not mapped into python
        if fn_name == "from" || fn_name == "as_ref" {
            return;
        }

        // If it has a new attribute must be treated as a constructor
        if is_constructor(fn_item) {
            write!(self.pyi_file, "\tdef __init__(self, ").unwrap();
        } else {
            write!(self.pyi_file, "\tdef {}(", fn_name).unwrap();
        }

        let mut is_first = true;

        for fn_arg in fn_item.sig.inputs.iter() {
            match fn_arg {
                syn::FnArg::Receiver(_) => {
                    write!(self.pyi_file, "self").unwrap();
                    is_first = false;
                }
                syn::FnArg::Typed(t) => {
                    if !is_first {
                        write!(self.pyi_file, ", ").unwrap();
                    }
                    let field_ident = match t.pat.as_ref() {
                        syn::Pat::Ident(i) => &i.ident,
                        _ => unreachable!(),
                    };
                    write!(self.pyi_file, "{}", field_ident).unwrap();
                    write!(self.pyi_file, ": ").unwrap();
                    write_type(self.pyi_file, t.ty.as_ref());
                    write_default_value(self.pyi_file, field_ident, &fn_item.attrs);

                    is_first = false;
                }
            }
        }

        write!(self.pyi_file, ") ").unwrap();

        if is_constructor(fn_item) {
            write!(self.pyi_file, "-> None").unwrap();
        } else {
            match &fn_item.sig.output {
                syn::ReturnType::Default => (),
                syn::ReturnType::Type(_, return_type) => {
                    write!(self.pyi_file, "-> ").unwrap();
                    write_type(self.pyi_file, return_type);
                }
            }
        }

        writeln!(self.pyi_file, " :").unwrap();

        for impl_block in self.dust_dds_ast_file.items.iter().filter_map(|i| match i {
            syn::Item::Impl(i) => Some(i),
            _ => None,
        }) {
            if let syn::Type::Path(p) = impl_block.self_ty.as_ref() {
                if p.path.segments[0].ident == *self.class_name {
                    if let Some(f) = impl_block
                        .items
                        .iter()
                        .filter_map(|i| match i {
                            syn::ImplItem::Fn(f) => Some(f),
                            _ => None,
                        })
                        .find(|f| f.sig.ident == fn_item.sig.ident)
                    {
                        writeln!(self.pyi_file, "\t\tr\"\"\"").unwrap();
                        write_docs(self.pyi_file, &f.attrs);
                        writeln!(self.pyi_file, "\t\t\"\"\"").unwrap();
                    }
                }
            }
        }

        writeln!(self.pyi_file, "\t\t...\n").unwrap();

        self.is_empty = false;
    }
}

impl<'ast> Visit<'ast> for PyiImplVisitor<'ast> {
    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        if let syn::Type::Path(impl_path) = i.self_ty.as_ref() {
            if let Some(path_ident) = impl_path.path.get_ident() {
                if *path_ident == self.class_name {
                    for fn_item in i.items.iter().filter_map(|i| match i {
                        syn::ImplItem::Fn(f) => Some(f),
                        _ => None,
                    }) {
                        self.write_fn_item(fn_item)
                    }
                }
            }
        }
    }
}

struct PyiStructVisitor<'ast> {
    pyi_file: &'ast mut fs::File,
    ast_file: &'ast syn::File,
    dust_dds_ast_file: &'ast syn::File,
}

impl<'ast> Visit<'ast> for PyiStructVisitor<'ast> {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        // Check if struct has attribute #[pyclass]
        if node
            .attrs
            .iter()
            .filter_map(|a| a.meta.path().get_ident())
            .any(|i| *i == "pyclass")
        {
            writeln!(self.pyi_file, "").unwrap();

            let class_name = node.ident.to_string();
            writeln!(self.pyi_file, "class {}:", class_name).unwrap();

            if let Some(s) = self
                .dust_dds_ast_file
                .items
                .iter()
                .filter_map(|i| match i {
                    syn::Item::Struct(s) => Some(s),
                    _ => None,
                })
                .find(|s| s.ident == node.ident)
            {
                writeln!(self.pyi_file, "\tr\"\"\"").unwrap();
                write_docs(self.pyi_file, &s.attrs);
                writeln!(self.pyi_file, "\t\"\"\"").unwrap();
            }
            let mut impl_visitor = PyiImplVisitor {
                pyi_file: self.pyi_file,
                class_name: node.ident.to_string(),
                is_empty: true,
                dust_dds_ast_file: self.dust_dds_ast_file,
            };
            impl_visitor.visit_file(self.ast_file);
            if impl_visitor.is_empty {
                writeln!(self.pyi_file, "\tpass\n").unwrap();
            }
        }
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        if i.attrs
            .iter()
            .filter_map(|a| a.meta.path().get_ident())
            .any(|i| *i == "pyclass")
        {
            let enum_name = i.ident.to_string();
            // We use only enum with unit and enum with named fields and all variants
            // must be the same so we match always on the first to decide the type
            match i.variants[0].fields {
                syn::Fields::Named(_) => {
                    // In case there are fields each variant represents a special class
                    for variant in i.variants.iter() {
                        writeln!(self.pyi_file, "\nclass {}_{}:", enum_name, variant.ident)
                            .unwrap();
                        if let Some(s) = self
                            .dust_dds_ast_file
                            .items
                            .iter()
                            .filter_map(|i| match i {
                                syn::Item::Enum(e) => Some(e),
                                _ => None,
                            })
                            .find(|e| e.ident == i.ident)
                        {
                            writeln!(self.pyi_file, "\tr\"\"\"").unwrap();
                            write_docs(self.pyi_file, &s.attrs);
                            writeln!(self.pyi_file, "\t\"\"\"").unwrap();
                        }
                        write!(self.pyi_file, "\tdef __init__(self").unwrap();
                        for field in variant.fields.iter() {
                            write!(self.pyi_file, ", ").unwrap();
                            write!(
                                self.pyi_file,
                                "{}",
                                field
                                    .ident
                                    .as_ref()
                                    .expect("All variant must have field ident")
                            )
                            .unwrap();
                            write!(self.pyi_file, ":").unwrap();
                            write_type(self.pyi_file, &field.ty);
                        }
                        write!(self.pyi_file, " ) -> None: ...").unwrap();
                    }
                    write!(self.pyi_file, "\nclass {}: \n", enum_name).unwrap();
                    for variant in i.variants.iter() {
                        writeln!(
                            self.pyi_file,
                            "\t{} = {}_{}",
                            variant.ident, enum_name, variant.ident
                        )
                        .unwrap();
                    }
                }
                syn::Fields::Unit => {
                    writeln!(self.pyi_file, "\nclass {}: ", enum_name).unwrap();
                    if let Some(s) = self
                        .dust_dds_ast_file
                        .items
                        .iter()
                        .filter_map(|i| match i {
                            syn::Item::Enum(e) => Some(e),
                            _ => None,
                        })
                        .find(|e| e.ident == i.ident)
                    {
                        writeln!(self.pyi_file, "\tr\"\"\"").unwrap();
                        write_docs(self.pyi_file, &s.attrs);
                        writeln!(self.pyi_file, "\t\"\"\"").unwrap();
                    }
                    for (idx, variant) in i.variants.iter().enumerate() {
                        writeln!(self.pyi_file, "\t{} = {}", variant.ident, idx).unwrap();
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

fn is_rs_file(file: &Path) -> bool {
    if let Some(extension) = file.extension() {
        extension == "rs"
    } else {
        false
    }
}

fn visit_fs_file(file: &Path, pyi_file: &mut File) -> io::Result<()> {
    if is_rs_file(file) {
        let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Variable should exist");
        let mut relative_file_path = file
            .strip_prefix(&cargo_dir)
            .expect("Path should have given prefix");
        if relative_file_path.starts_with("src") {
            relative_file_path = relative_file_path
                .strip_prefix("src")
                .expect("Starts with src");
        }
        let dust_dds_dir = Path::new(&cargo_dir).join("../../dds/src/dds");
        let dust_dds_file_path = dust_dds_dir.join(relative_file_path);

        let mut file = File::open(file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut dust_dds_content = String::new();
        if let Ok(mut dust_dds_file) = File::open(dust_dds_file_path) {
            dust_dds_file.read_to_string(&mut dust_dds_content)?;
        }

        let ast = syn::parse_file(&content).expect("File should be valid");
        let dust_dds_ast = syn::parse_file(&dust_dds_content).expect("File should be valid");

        PyiStructVisitor {
            pyi_file,
            ast_file: &ast,
            dust_dds_ast_file: &dust_dds_ast,
        }
        .visit_file(&ast);
    }
    Ok(())
}

fn visit_fs_dir(dir: &Path, pyi_file: &mut File) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_fs_dir(&path, pyi_file)?;
            } else if path.is_file() {
                visit_fs_file(&path, pyi_file)?;
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let cargo_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Variable should exist");
    let cargo_dir_path = Path::new(&cargo_dir);
    let mut pyi_file = File::create(cargo_dir_path.join("dust_dds.pyi"))?;

    writeln!(pyi_file, "from typing import Any \n").unwrap();

    // Add the constants manually
    writeln!(pyi_file, "ANY_SAMPLE_STATE : SampleStateKind= ...").unwrap();
    writeln!(pyi_file, "ANY_VIEW_STATE : ViewStateKind = ...").unwrap();
    writeln!(pyi_file, "ANY_INSTANCE_STATE : InstanceStateKind= ...").unwrap();
    writeln!(
        pyi_file,
        "NOT_ALIVE_INSTANCE_STATE : InstanceStateKind= ..."
    )
    .unwrap();
    writeln!(
        pyi_file,
        "DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS : ReliabilityQosPolicy = ..."
    )
    .unwrap();
    writeln!(
        pyi_file,
        "DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER : ReliabilityQosPolicy = ..."
    )
    .unwrap();

    visit_fs_dir(cargo_dir_path, &mut pyi_file)?;

    Ok(())
}
