use std::fmt::{self, Display};

use super::{
  net_to_term::{ReadbackError, ReadbackErrors},
  Book, DefName, Definition, MatchNum, Name, Op, Pattern, Rule, Tag, Term, Type,
};

macro_rules! display {
  ($($x:tt)*) => {
    DisplayFn(move |f| write!(f, $($x)*))
  };
}

impl Term {
  fn display_app<'a>(&'a self, tag: &'a Tag) -> impl Display + 'a {
    DisplayFn(move |f| match self {
      Term::App { tag: tag2, fun, arg } if tag2 == tag => {
        write!(f, "{} {}", fun.display_app(tag), arg.display())
      }
      _ => write!(f, "{}", self.display()),
    })
  }
  pub fn display(&self) -> impl Display + '_ {
    DisplayFn(move |f| match self {
      Term::Lam { tag, nam, bod } => {
        write!(f, "{}λ{} {}", tag.display_padded(), nam.as_ref().map_or("*", |x| x.as_str()), bod.display())
      }
      Term::Var { nam } => write!(f, "{nam}"),
      Term::Chn { tag, nam, bod } => {
        write!(f, "{}λ${} {}", tag.display_padded(), nam, bod.display())
      }
      Term::Lnk { nam } => write!(f, "${nam}"),
      Term::Let { pat, val, nxt } => {
        write!(f, "let {} = {}; {}", pat, val.display(), nxt.display())
      }
      Term::Ref { def_name } => write!(f, "{def_name}"),
      Term::App { tag, fun, arg } => {
        write!(f, "{}({} {})", tag.display_padded(), fun.display_app(tag), arg.display())
      }
      Term::Match { scrutinee, arms } => {
        write!(
          f,
          "match {} {{ {} }}",
          scrutinee.display(),
          DisplayJoin(|| arms.iter().map(|(pat, term)| display!("{}: {}", pat, term.display())), "; "),
        )
      }
      Term::Dup { tag, fst, snd, val, nxt } => write!(
        f,
        "let {}{{{} {}}} = {}; {}",
        tag.display(),
        fst.as_ref().map_or("*", |x| x.as_str()),
        snd.as_ref().map_or("*", |x| x.as_str()),
        val.display(),
        nxt.display()
      ),
      Term::Sup { tag, fst, snd } => {
        write!(f, "{}{{{} {}}}", tag.display(), fst.display(), snd.display())
      }
      Term::Era => write!(f, "*"),
      Term::Num { val } => write!(f, "{val}"),
      Term::Str { val } => write!(f, "{val:?}"),
      Term::Opx { op, fst, snd } => {
        write!(f, "({} {} {})", op, fst.display(), snd.display())
      }
      Term::Tup { fst, snd } => write!(f, "({}, {})", fst.display(), snd.display()),
      Term::List { els } => {
        write!(f, "[{}]", DisplayJoin(|| els.iter().map(|el| display!("{}", el.display())), ", "),)
      }
      Term::Invalid => write!(f, "<Invalid>"),
    })
  }
}

impl Tag {
  pub fn display_padded(&self) -> impl Display + '_ {
    DisplayFn(move |f| match self {
      Tag::Named(name) => write!(f, "#{name} "),
      Tag::Numeric(num) => write!(f, "#{num} "),
      Tag::Auto => Ok(()),
      Tag::Static => Ok(()),
    })
  }
  pub fn display(&self) -> impl Display + '_ {
    DisplayFn(move |f| match self {
      Tag::Named(name) => write!(f, "#{name}"),
      Tag::Numeric(num) => write!(f, "#{num}"),
      Tag::Auto => Ok(()),
      Tag::Static => Ok(()),
    })
  }
}

impl fmt::Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Pattern::Var(None) => write!(f, "*"),
      Pattern::Var(Some(nam)) => write!(f, "{nam}"),
      Pattern::Ctr(nam, pats) => {
        write!(f, "({}{})", nam, DisplayJoin(|| pats.iter().map(|p| display!(" {p}")), ""))
      }
      Pattern::Num(num) => write!(f, "{num}"),
      Pattern::Tup(fst, snd) => write!(f, "({}, {})", fst, snd,),
      Pattern::List(pats) => write!(f, "[{}]", DisplayJoin(|| pats.iter().map(|p| display!("{p}")), ", ")),
    }
  }
}

impl Rule {
  pub fn display<'a>(&'a self, def_name: &'a DefName) -> impl Display + 'a {
    display!(
      "({}{}) = {}",
      def_name,
      DisplayJoin(|| self.pats.iter().map(|x| display!(" {x}")), ""),
      self.body.display()
    )
  }
}

impl Definition {
  pub fn display(&self) -> impl Display + '_ {
    DisplayJoin(|| self.rules.iter().map(|x| x.display(&self.name)), "\n")
  }
}

impl fmt::Display for Book {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", DisplayJoin(|| self.defs.values().map(|x| x.display()), "\n\n"))
  }
}

impl fmt::Display for MatchNum {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      MatchNum::Zero => write!(f, "0"),
      MatchNum::Succ(None) => write!(f, "+"),
      MatchNum::Succ(Some(None)) => write!(f, "+*"),
      MatchNum::Succ(Some(Some(nam))) => write!(f, "+{nam}"),
    }
  }
}

impl fmt::Display for Op {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Op::ADD => write!(f, "+"),
      Op::SUB => write!(f, "-"),
      Op::MUL => write!(f, "*"),
      Op::DIV => write!(f, "/"),
      Op::MOD => write!(f, "%"),
      Op::EQ => write!(f, "=="),
      Op::NE => write!(f, "!="),
      Op::LT => write!(f, "<"),
      Op::GT => write!(f, ">"),
      Op::LTE => write!(f, "<="),
      Op::GTE => write!(f, ">="),
      Op::AND => write!(f, "&"),
      Op::OR => write!(f, "|"),
      Op::XOR => write!(f, "^"),
      Op::LSH => write!(f, "<<"),
      Op::RSH => write!(f, ">>"),
      Op::NOT => write!(f, "~"),
    }
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Type::Any => write!(f, "any"),
      Type::Tup => write!(f, "tup"),
      Type::Num => write!(f, "num"),
      Type::Adt(nam) => write!(f, "{nam}"),
      Type::None => unreachable!(),
    }
  }
}
struct DisplayFn<F: Fn(&mut fmt::Formatter) -> fmt::Result>(F);

impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> Display for DisplayFn<F> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0(f)
  }
}

pub struct DisplayJoin<F, S>(pub F, pub S);

impl<F, I, S> Display for DisplayJoin<F, S>
where
  F: (Fn() -> I),
  I: Iterator,
  I::Item: Display,
  S: Display,
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, x) in self.0().enumerate() {
      if i != 0 {
        self.1.fmt(f)?;
      }
      x.fmt(f)?;
    }
    Ok(())
  }
}

impl ReadbackErrors {
  pub fn display(&self) -> impl fmt::Display + '_ {
    DisplayFn(move |f| {
      if self.0.is_empty() {
        return Ok(());
      }

      writeln!(f, "Readback Warning:")?;
      let mut err_counts = std::collections::HashMap::new();
      for err in &self.0 {
        if err.can_count() {
          *err_counts.entry(err).or_insert(0) += 1;
        } else {
          writeln!(f, "{}", err.display())?;
        }
      }

      for (err, count) in err_counts {
        write!(f, "{}", err.display())?;
        if count > 1 {
          writeln!(f, " with {} occurrences", count)?;
        }
      }

      writeln!(f)
    })
  }
}

impl ReadbackError {
  pub fn display(&self) -> impl Display + '_ {
    DisplayFn(move |f| match self {
      ReadbackError::InvalidNumericMatch => write!(f, "Invalid Numeric Match"),
      ReadbackError::InvalidNumericOp => write!(f, "Invalid Numeric Operation"),
      ReadbackError::ReachedRoot => write!(f, "Reached Root"),
      ReadbackError::Cyclic => write!(f, "Cyclic Term"),
      ReadbackError::InvalidBind => write!(f, "Invalid Bind"),
      ReadbackError::InvalidAdt => write!(f, "Invalid Adt"),
      ReadbackError::UnexpectedTag(exp, fnd) => {
        write!(f, "Unexpected tag found during Adt readback, expected '{}', but found ", exp.display())?;

        match fnd {
          Tag::Static => write!(f, "no tag"),
          _ => write!(f, "'{}'", fnd.display()),
        }
      }
      ReadbackError::InvalidAdtMatch => write!(f, "Invalid Adt Match"),
      ReadbackError::InvalidStrTerm(term) => {
        write!(f, "Invalid String Character value '{}'", term.display())
      }
    })
  }
}
