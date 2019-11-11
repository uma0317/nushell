use crate::prelude::*;
use derive_new::new;
use getset::Getters;
use std::fmt::{self, Write};

pub trait ShellTypeName {
    fn type_name(&self) -> &'static str;
}

impl<T: ShellTypeName> ShellTypeName for &T {
    fn type_name(&self) -> &'static str {
        (*self).type_name()
    }
}

pub trait SpannedTypeName {
    fn spanned_type_name(&self) -> Spanned<&'static str>;
}

impl<T: ShellTypeName> SpannedTypeName for Spanned<T> {
    fn spanned_type_name(&self) -> Spanned<&'static str> {
        self.item.type_name().spanned(self.span)
    }
}

pub struct Debuggable<'a, T: FormatDebug> {
    inner: &'a T,
    source: &'a str,
}

impl FormatDebug for str {
    fn fmt_debug(&self, f: &mut DebugFormatter, _source: &str) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T: ToDebug> fmt::Debug for Debuggable<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt_debug(
            &mut DebugFormatter::new(
                f,
                ansi_term::Color::White.bold(),
                ansi_term::Color::Black.bold(),
            ),
            self.source,
        )
    }
}

impl<T: ToDebug> fmt::Display for Debuggable<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt_display(
            &mut DebugFormatter::new(
                f,
                ansi_term::Color::White.bold(),
                ansi_term::Color::Black.bold(),
            ),
            self.source,
        )
    }
}

pub trait HasTag {
    fn tag(&self) -> Tag;
}

#[derive(Getters, new)]
pub struct DebugFormatter<'me, 'args> {
    formatter: &'me mut std::fmt::Formatter<'args>,
    #[new(value = "pretty::Arena::new()")]
    #[get = "pub"]
    arena: pretty::Arena<'me, DebugDoc<'me>>,
    style: ansi_term::Style,
    default_style: ansi_term::Style,
}

impl<'me, 'args> DebugFormatter<'me, 'args> {
    pub fn say_simple(&mut self, kind: &str) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))
    }

    pub fn say<'debuggable>(
        &mut self,
        kind: &str,
        debuggable: Debuggable<'debuggable, impl FormatDebug>,
    ) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))?;
        write!(self, "{}", self.default_style.paint(" "))?;
        write!(
            self,
            "{}",
            self.default_style.paint(format!("{}", debuggable))
        )
    }

    pub fn say_str<'debuggable>(
        &mut self,
        kind: &str,
        string: impl AsRef<str>,
    ) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))?;
        write!(self, "{}", self.default_style.paint(" "))?;
        write!(self, "{}", self.default_style.paint(string.as_ref()))
    }

    pub fn say_block(
        &mut self,
        kind: &str,
        block: impl FnOnce(&mut Self) -> std::fmt::Result,
    ) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))?;
        write!(self, "{}", self.default_style.paint(" "))?;
        block(self)
    }

    pub fn say_list<T, U: IntoIterator<Item = T>>(
        &mut self,
        kind: &str,
        list: U,
        open: impl Fn(&mut Self) -> std::fmt::Result,
        mut block: impl FnMut(&mut Self, &T) -> std::fmt::Result,
        interleave: impl Fn(&mut Self) -> std::fmt::Result,
        close: impl Fn(&mut Self) -> std::fmt::Result,
    ) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))?;
        write!(self, "{}", self.default_style.paint(" "))?;
        open(self)?;
        write!(self, " ")?;

        let mut list = list.into_iter();

        let first = match list.next() {
            None => return Ok(()),
            Some(first) => first,
        };

        block(self, &first)?;

        for item in list {
            interleave(self)?;
            block(self, &item)?;
        }

        write!(self, " ")?;
        close(self)?;

        Ok(())
    }

    pub fn say_dict<'debuggable>(
        &mut self,
        kind: &str,
        dict: indexmap::IndexMap<&str, String>,
    ) -> std::fmt::Result {
        write!(self, "{}", self.style.paint(kind))?;
        write!(self, "{}", self.default_style.paint(" "))?;

        let last = dict.len() - 1;

        for (i, (key, value)) in dict.into_iter().enumerate() {
            write!(self, "{}", self.default_style.paint(key))?;
            write!(self, "{}", self.default_style.paint("=["))?;
            write!(self, "{}", self.style.paint(value))?;
            write!(self, "{}", self.default_style.paint("]"))?;

            if i != last {
                write!(self, "{}", self.default_style.paint(" "))?;
            }
        }

        Ok(())
    }
}

impl<'a, 'b> std::fmt::Write for DebugFormatter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.formatter.write_str(s)
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.formatter.write_char(c)
    }

    fn write_fmt(self: &mut Self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.formatter.write_fmt(args)
    }
}

pub struct ShellAnnotation {}

pub type DebugDoc<'a> = pretty::RefDoc<'a, ShellAnnotation>;

pub trait PrettyDebug {
    fn pretty_debug<'arena>(&self, f: &'arena mut DebugFormatter, source: &str)
        -> DebugDoc<'arena>;
}

pub trait FormatDebug: std::fmt::Debug {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result;

    fn fmt_display(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        self.fmt_debug(f, source)
    }
}

pub trait ToDebug: Sized + FormatDebug {
    fn debug<'a>(&'a self, source: &'a str) -> Debuggable<'a, Self>;
}

impl FormatDebug for Box<dyn FormatDebug> {
    fn fmt_debug(&self, f: &mut DebugFormatter, source: &str) -> fmt::Result {
        (&**self).fmt_debug(f, source)
    }
}

impl<T> ToDebug for T
where
    T: FormatDebug + Sized,
{
    fn debug<'a>(&'a self, source: &'a str) -> Debuggable<'a, Self> {
        Debuggable {
            inner: self,
            source,
        }
    }
}
