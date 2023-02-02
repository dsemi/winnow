//! Parsers recognizing bytes streams, complete input version

#![allow(deprecated)]

use crate::error::ErrMode;
use crate::error::ErrorKind;
use crate::error::ParseError;
use crate::input::{
  split_at_offset1_complete, split_at_offset_complete, Compare, CompareResult, ContainsToken,
  FindSlice, Input, Offset, SliceLen, ToUsize,
};
use crate::lib::std::result::Result::Ok;
use crate::{IResult, Parser};

pub(crate) fn any<I, E: ParseError<I>>(input: I) -> IResult<I, <I as Input>::Token, E>
where
  I: Input,
{
  input
    .next_token()
    .ok_or_else(|| ErrMode::Backtrack(E::from_error_kind(input, ErrorKind::Eof)))
}

/// Recognizes a pattern
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument
///
/// It will return `Err(ErrMode::Backtrack((_, ErrorKind::Tag)))` if the input doesn't match the pattern
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::tag;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag("Hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("Something"), Err(ErrMode::Backtrack(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::tag`][crate::bytes::tag]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::tag`")]
pub fn tag<T, I, Error: ParseError<I>>(
  tag: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Compare<T>,
  T: SliceLen + Clone,
{
  move |i: I| tag_internal(i, tag.clone())
}

pub(crate) fn tag_internal<T, I, Error: ParseError<I>>(
  i: I,
  t: T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Compare<T>,
  T: SliceLen,
{
  let tag_len = t.slice_len();
  match i.compare(t) {
    CompareResult::Ok => Ok(i.next_slice(tag_len)),
    CompareResult::Incomplete | CompareResult::Error => {
      let e: ErrorKind = ErrorKind::Tag;
      Err(ErrMode::Backtrack(Error::from_error_kind(i, e)))
    }
  }
}

/// Recognizes a case insensitive pattern.
///
/// The input data will be compared to the tag combinator's argument and will return the part of
/// the input that matches the argument with no regard to case.
///
/// It will return `Err(ErrMode::Backtrack((_, ErrorKind::Tag)))` if the input doesn't match the pattern.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::tag_no_case;
///
/// fn parser(s: &str) -> IResult<&str, &str> {
///   tag_no_case("hello")(s)
/// }
///
/// assert_eq!(parser("Hello, World!"), Ok((", World!", "Hello")));
/// assert_eq!(parser("hello, World!"), Ok((", World!", "hello")));
/// assert_eq!(parser("HeLlO, World!"), Ok((", World!", "HeLlO")));
/// assert_eq!(parser("Something"), Err(ErrMode::Backtrack(Error::new("Something", ErrorKind::Tag))));
/// assert_eq!(parser(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Tag))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::tag_no_case`][crate::bytes::tag_no_case]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::tag_no_case`")]
pub fn tag_no_case<T, I, Error: ParseError<I>>(
  tag: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Compare<T>,
  T: SliceLen + Clone,
{
  move |i: I| tag_no_case_internal(i, tag.clone())
}

pub(crate) fn tag_no_case_internal<T, I, Error: ParseError<I>>(
  i: I,
  t: T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Compare<T>,
  T: SliceLen,
{
  let tag_len = t.slice_len();

  match (i).compare_no_case(t) {
    CompareResult::Ok => Ok(i.next_slice(tag_len)),
    CompareResult::Incomplete | CompareResult::Error => {
      let e: ErrorKind = ErrorKind::Tag;
      Err(ErrMode::Backtrack(Error::from_error_kind(i, e)))
    }
  }
}

pub(crate) fn one_of_internal<I, T, E: ParseError<I>>(
  input: I,
  list: &T,
) -> IResult<I, <I as Input>::Token, E>
where
  I: Input,
  <I as Input>::Token: Copy,
  T: ContainsToken<<I as Input>::Token>,
{
  input
    .next_token()
    .filter(|(_, t)| list.contains_token(*t))
    .ok_or_else(|| ErrMode::Backtrack(E::from_error_kind(input, ErrorKind::OneOf)))
}

pub(crate) fn none_of_internal<I, T, E: ParseError<I>>(
  input: I,
  list: &T,
) -> IResult<I, <I as Input>::Token, E>
where
  I: Input,
  <I as Input>::Token: Copy,
  T: ContainsToken<<I as Input>::Token>,
{
  input
    .next_token()
    .filter(|(_, t)| !list.contains_token(*t))
    .ok_or_else(|| ErrMode::Backtrack(E::from_error_kind(input, ErrorKind::NoneOf)))
}

/// Parse till certain characters are met.
///
/// The parser will return the longest slice till one of the characters of the combinator's argument are met.
///
/// It doesn't consume the matched character.
///
/// It will return a `ErrMode::Backtrack(("", ErrorKind::IsNot))` if the pattern wasn't met.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::is_not;
///
/// fn not_space(s: &str) -> IResult<&str, &str> {
///   is_not(" \t\r\n")(s)
/// }
///
/// assert_eq!(not_space("Hello, World!"), Ok((" World!", "Hello,")));
/// assert_eq!(not_space("Sometimes\t"), Ok(("\t", "Sometimes")));
/// assert_eq!(not_space("Nospace"), Ok(("", "Nospace")));
/// assert_eq!(not_space(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::IsNot))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_till1`][crate::bytes::take_till1]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_till1`")]
pub fn is_not<T, I, Error: ParseError<I>>(
  arr: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| is_not_internal(i, &arr)
}

pub(crate) fn is_not_internal<T, I, Error: ParseError<I>>(
  i: I,
  arr: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  let e: ErrorKind = ErrorKind::IsNot;
  split_at_offset1_complete(&i, |c| arr.contains_token(c), e)
}

/// Returns the longest slice of the matches the pattern.
///
/// The parser will return the longest slice consisting of the characters in provided in the
/// combinator's argument.
///
/// It will return a `Err(ErrMode::Backtrack((_, ErrorKind::IsA)))` if the pattern wasn't met.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::is_a;
///
/// fn hex(s: &str) -> IResult<&str, &str> {
///   is_a("1234567890ABCDEF")(s)
/// }
///
/// assert_eq!(hex("123 and voila"), Ok((" and voila", "123")));
/// assert_eq!(hex("DEADBEEF and others"), Ok((" and others", "DEADBEEF")));
/// assert_eq!(hex("BADBABEsomething"), Ok(("something", "BADBABE")));
/// assert_eq!(hex("D15EA5E"), Ok(("", "D15EA5E")));
/// assert_eq!(hex(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::IsA))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_while1`][crate::bytes::take_while1`]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_while1`")]
pub fn is_a<T, I, Error: ParseError<I>>(
  arr: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| is_a_internal(i, &arr)
}

pub(crate) fn is_a_internal<T, I, Error: ParseError<I>>(
  i: I,
  arr: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  let e: ErrorKind = ErrorKind::IsA;
  split_at_offset1_complete(&i, |c| !arr.contains_token(c), e)
}

/// Returns the longest input slice (if any) that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::bytes::complete::take_while;
/// use winnow::input::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Ok((&b"12345"[..], &b""[..])));
/// assert_eq!(alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(alpha(b""), Ok((&b""[..], &b""[..])));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_while`][crate::bytes::take_while]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_while`")]
pub fn take_while<T, I, Error: ParseError<I>>(
  list: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| take_while_internal(i, &list)
}

pub(crate) fn take_while_internal<T, I, Error: ParseError<I>>(
  i: I,
  list: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  split_at_offset_complete(&i, |c| !list.contains_token(c))
}

/// Returns the longest (at least 1) input slice that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `Err(ErrMode::Backtrack((_, ErrorKind::TakeWhile1)))` if the pattern wasn't met.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take_while1;
/// use winnow::input::AsChar;
///
/// fn alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while1(AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(alpha(b"12345"), Err(ErrMode::Backtrack(Error::new(&b"12345"[..], ErrorKind::TakeWhile1))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_while1`][crate::bytes::take_while1]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_while1`")]
pub fn take_while1<T, I, Error: ParseError<I>>(
  list: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| take_while1_internal(i, &list)
}

pub(crate) fn take_while1_internal<T, I, Error: ParseError<I>>(
  i: I,
  list: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  let e: ErrorKind = ErrorKind::TakeWhile1;
  split_at_offset1_complete(&i, |c| !list.contains_token(c), e)
}

/// Returns the longest (m <= len <= n) input slice  that matches the predicate.
///
/// The parser will return the longest slice that matches the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return an `ErrMode::Backtrack((_, ErrorKind::TakeWhileMN))` if the pattern wasn't met or is out
/// of range (m <= len <= n).
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take_while_m_n;
/// use winnow::input::AsChar;
///
/// fn short_alpha(s: &[u8]) -> IResult<&[u8], &[u8]> {
///   take_while_m_n(3, 6, AsChar::is_alpha)(s)
/// }
///
/// assert_eq!(short_alpha(b"latin123"), Ok((&b"123"[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"lengthy"), Ok((&b"y"[..], &b"length"[..])));
/// assert_eq!(short_alpha(b"latin"), Ok((&b""[..], &b"latin"[..])));
/// assert_eq!(short_alpha(b"ed"), Err(ErrMode::Backtrack(Error::new(&b"ed"[..], ErrorKind::TakeWhileMN))));
/// assert_eq!(short_alpha(b"12345"), Err(ErrMode::Backtrack(Error::new(&b"12345"[..], ErrorKind::TakeWhileMN))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_while_m_n`][crate::bytes::take_while_m_n]
#[deprecated(
  since = "8.0.0",
  note = "Replaced with `winnow::bytes::take_while_m_n`"
)]
pub fn take_while_m_n<T, I, Error: ParseError<I>>(
  m: usize,
  n: usize,
  list: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| take_while_m_n_internal(i, m, n, &list)
}

pub(crate) fn take_while_m_n_internal<T, I, Error: ParseError<I>>(
  input: I,
  m: usize,
  n: usize,
  list: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  match input.offset_for(|c| !list.contains_token(c)) {
    Some(idx) => {
      if idx >= m {
        if idx <= n {
          let res: IResult<_, _, Error> = if let Ok(index) = input.offset_at(idx) {
            Ok(input.next_slice(index))
          } else {
            Err(ErrMode::Backtrack(Error::from_error_kind(
              input,
              ErrorKind::TakeWhileMN,
            )))
          };
          res
        } else {
          let res: IResult<_, _, Error> = if let Ok(index) = input.offset_at(n) {
            Ok(input.next_slice(index))
          } else {
            Err(ErrMode::Backtrack(Error::from_error_kind(
              input,
              ErrorKind::TakeWhileMN,
            )))
          };
          res
        }
      } else {
        let e = ErrorKind::TakeWhileMN;
        Err(ErrMode::Backtrack(Error::from_error_kind(input, e)))
      }
    }
    None => {
      let len = input.input_len();
      if len >= n {
        match input.offset_at(n) {
          Ok(index) => Ok(input.next_slice(index)),
          Err(_needed) => Err(ErrMode::Backtrack(Error::from_error_kind(
            input,
            ErrorKind::TakeWhileMN,
          ))),
        }
      } else if len >= m && len <= n {
        Ok(input.next_slice(len))
      } else {
        let e = ErrorKind::TakeWhileMN;
        Err(ErrMode::Backtrack(Error::from_error_kind(input, e)))
      }
    }
  }
}

/// Returns the longest input slice (if any) till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// use winnow::bytes::complete::take_till;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Ok((":empty matched", ""))); //allowed
/// assert_eq!(till_colon("12345"), Ok(("", "12345")));
/// assert_eq!(till_colon(""), Ok(("", "")));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_till`][crate::bytes::take_till]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_till`")]
#[allow(clippy::redundant_closure)]
pub fn take_till<T, I, Error: ParseError<I>>(
  list: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| take_till_internal(i, &list)
}

pub(crate) fn take_till_internal<T, I, Error: ParseError<I>>(
  i: I,
  list: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  split_at_offset_complete(&i, |c| list.contains_token(c))
}

/// Returns the longest (at least 1) input slice till a predicate is met.
///
/// The parser will return the longest slice till the given predicate *(a function that
/// takes the input and returns a bool)*.
///
/// It will return `Err(ErrMode::Backtrack((_, ErrorKind::TakeTill1)))` if the input is empty or the
/// predicate matches the first input.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take_till1;
///
/// fn till_colon(s: &str) -> IResult<&str, &str> {
///   take_till1(|c| c == ':')(s)
/// }
///
/// assert_eq!(till_colon("latin:123"), Ok((":123", "latin")));
/// assert_eq!(till_colon(":empty matched"), Err(ErrMode::Backtrack(Error::new(":empty matched", ErrorKind::TakeTill1))));
/// assert_eq!(till_colon("12345"), Ok(("", "12345")));
/// assert_eq!(till_colon(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::TakeTill1))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_till1`][crate::bytes::take_till1]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_till1`")]
#[allow(clippy::redundant_closure)]
pub fn take_till1<T, I, Error: ParseError<I>>(
  list: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  move |i: I| take_till1_internal(i, &list)
}

pub(crate) fn take_till1_internal<T, I, Error: ParseError<I>>(
  i: I,
  list: &T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  T: ContainsToken<<I as Input>::Token>,
{
  let e: ErrorKind = ErrorKind::TakeTill1;
  split_at_offset1_complete(&i, |c| list.contains_token(c), e)
}

/// Returns an input slice containing the first N input elements (I[..N]).
///
/// It will return `Err(ErrMode::Backtrack((_, ErrorKind::Eof)))` if the input is shorter than the argument.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take;
///
/// fn take6(s: &str) -> IResult<&str, &str> {
///   take(6usize)(s)
/// }
///
/// assert_eq!(take6("1234567"), Ok(("7", "123456")));
/// assert_eq!(take6("things"), Ok(("", "things")));
/// assert_eq!(take6("short"), Err(ErrMode::Backtrack(Error::new("short", ErrorKind::Eof))));
/// assert_eq!(take6(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::Eof))));
/// ```
///
/// The units that are taken will depend on the input type. For example, for a
/// `&str` it will take a number of `char`'s, whereas for a `&[u8]` it will
/// take that many `u8`'s:
///
/// ```rust
/// use winnow::error::Error;
/// use winnow::bytes::complete::take;
///
/// assert_eq!(take::<_, _, Error<_>>(1usize)("💙"), Ok(("", "💙")));
/// assert_eq!(take::<_, _, Error<_>>(1usize)("💙".as_bytes()), Ok((b"\x9F\x92\x99".as_ref(), b"\xF0".as_ref())));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take`][crate::bytes::take]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take`")]
pub fn take<C, I, Error: ParseError<I>>(
  count: C,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
  C: ToUsize,
{
  let c = count.to_usize();
  move |i: I| take_internal(i, c)
}

pub(crate) fn take_internal<I, Error: ParseError<I>>(
  i: I,
  c: usize,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input,
{
  match i.offset_at(c) {
    Ok(offset) => Ok(i.next_slice(offset)),
    Err(_needed) => Err(ErrMode::Backtrack(Error::from_error_kind(
      i,
      ErrorKind::Eof,
    ))),
  }
}

/// Returns the input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern. It will return `Err(ErrMode::Backtrack((_, ErrorKind::TakeUntil)))`
/// if the pattern wasn't met.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take_until;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until("eof")(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(ErrMode::Backtrack(Error::new("hello, world", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_until`][crate::bytes::take_until]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_until`")]
pub fn take_until<T, I, Error: ParseError<I>>(
  tag: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + FindSlice<T>,
  T: SliceLen + Clone,
{
  move |i: I| take_until_internal(i, tag.clone())
}

pub(crate) fn take_until_internal<T, I, Error: ParseError<I>>(
  i: I,
  t: T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + FindSlice<T>,
  T: SliceLen,
{
  match i.find_slice(t) {
    Some(offset) => Ok(i.next_slice(offset)),
    None => Err(ErrMode::Backtrack(Error::from_error_kind(
      i,
      ErrorKind::TakeUntil,
    ))),
  }
}

/// Returns the non empty input slice up to the first occurrence of the pattern.
///
/// It doesn't consume the pattern. It will return `Err(ErrMode::Backtrack((_, ErrorKind::TakeUntil)))`
/// if the pattern wasn't met.
/// # Example
/// ```rust
/// # use winnow::{error::ErrMode, error::{Error, ErrorKind}, error::Needed, IResult};
/// use winnow::bytes::complete::take_until1;
///
/// fn until_eof(s: &str) -> IResult<&str, &str> {
///   take_until1("eof")(s)
/// }
///
/// assert_eq!(until_eof("hello, worldeof"), Ok(("eof", "hello, world")));
/// assert_eq!(until_eof("hello, world"), Err(ErrMode::Backtrack(Error::new("hello, world", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof(""), Err(ErrMode::Backtrack(Error::new("", ErrorKind::TakeUntil))));
/// assert_eq!(until_eof("1eof2eof"), Ok(("eof2eof", "1")));
/// assert_eq!(until_eof("eof"), Err(ErrMode::Backtrack(Error::new("eof", ErrorKind::TakeUntil))));
/// ```
///
/// **WARNING:** Deprecated, replaced with [`winnow::bytes::take_until1`][crate::bytes::take_until1]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::bytes::take_until1`")]
pub fn take_until1<T, I, Error: ParseError<I>>(
  tag: T,
) -> impl Fn(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + FindSlice<T>,
  T: SliceLen + Clone,
{
  move |i: I| take_until1_internal(i, tag.clone())
}

pub(crate) fn take_until1_internal<T, I, Error: ParseError<I>>(
  i: I,
  t: T,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + FindSlice<T>,
  T: SliceLen,
{
  match i.find_slice(t) {
    None | Some(0) => Err(ErrMode::Backtrack(Error::from_error_kind(
      i,
      ErrorKind::TakeUntil,
    ))),
    Some(offset) => Ok(i.next_slice(offset)),
  }
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not accept the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters
/// # Example
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// # use winnow::character::complete::digit1;
/// use winnow::bytes::complete::escaped;
/// use winnow::character::complete::one_of;
///
/// fn esc(s: &str) -> IResult<&str, &str> {
///   escaped(digit1, '\\', one_of(r#""n\"#))(s)
/// }
///
/// assert_eq!(esc("123;"), Ok((";", "123")));
/// assert_eq!(esc(r#"12\"34;"#), Ok((";", r#"12\"34"#)));
/// ```
///
///
/// **WARNING:** Deprecated, replaced with [`winnow::character::escaped`][crate::character::escaped]
#[deprecated(since = "8.0.0", note = "Replaced with `winnow::character::escaped`")]
pub fn escaped<'a, I: 'a, Error, F, G, O1, O2>(
  mut normal: F,
  control_char: char,
  mut escapable: G,
) -> impl FnMut(I) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Offset,
  <I as Input>::Token: crate::input::AsChar,
  F: Parser<I, O1, Error>,
  G: Parser<I, O2, Error>,
  Error: ParseError<I>,
{
  move |input: I| escaped_internal(input, &mut normal, control_char, &mut escapable)
}

pub(crate) fn escaped_internal<'a, I: 'a, Error, F, G, O1, O2>(
  input: I,
  normal: &mut F,
  control_char: char,
  escapable: &mut G,
) -> IResult<I, <I as Input>::Slice, Error>
where
  I: Input + Offset,
  <I as Input>::Token: crate::input::AsChar,
  F: Parser<I, O1, Error>,
  G: Parser<I, O2, Error>,
  Error: ParseError<I>,
{
  use crate::input::AsChar;

  let mut i = input.clone();

  while i.input_len() > 0 {
    let current_len = i.input_len();

    match normal.parse_next(i.clone()) {
      Ok((i2, _)) => {
        // return if we consumed everything or if the normal parser
        // does not consume anything
        if i2.input_len() == 0 {
          return Ok(input.next_slice(input.input_len()));
        } else if i2.input_len() == current_len {
          let offset = input.offset_to(&i2);
          return Ok(input.next_slice(offset));
        } else {
          i = i2;
        }
      }
      Err(ErrMode::Backtrack(_)) => {
        if i.next_token().expect("input_len > 0").1.as_char() == control_char {
          let next = control_char.len_utf8();
          if next >= i.input_len() {
            return Err(ErrMode::Backtrack(Error::from_error_kind(
              input,
              ErrorKind::Escaped,
            )));
          } else {
            match escapable.parse_next(i.next_slice(next).0) {
              Ok((i2, _)) => {
                if i2.input_len() == 0 {
                  return Ok(input.next_slice(input.input_len()));
                } else {
                  i = i2;
                }
              }
              Err(e) => return Err(e),
            }
          }
        } else {
          let offset = input.offset_to(&i);
          if offset == 0 {
            return Err(ErrMode::Backtrack(Error::from_error_kind(
              input,
              ErrorKind::Escaped,
            )));
          }
          return Ok(input.next_slice(offset));
        }
      }
      Err(e) => {
        return Err(e);
      }
    }
  }

  Ok(input.next_slice(input.input_len()))
}

/// Matches a byte string with escaped characters.
///
/// * The first argument matches the normal characters (it must not match the control character)
/// * The second argument is the control character (like `\` in most languages)
/// * The third argument matches the escaped characters and transforms them
///
/// As an example, the chain `abc\tdef` could be `abc    def` (it also consumes the control character)
///
/// ```
/// # use winnow::{error::ErrMode, error::ErrorKind, error::Needed, IResult};
/// # use std::str::from_utf8;
/// use winnow::bytes::complete::{escaped_transform, tag};
/// use winnow::character::complete::alpha1;
/// use winnow::branch::alt;
/// use winnow::combinator::value;
///
/// fn parser(input: &str) -> IResult<&str, String> {
///   escaped_transform(
///     alpha1,
///     '\\',
///     alt((
///       value("\\", tag("\\")),
///       value("\"", tag("\"")),
///       value("\n", tag("n")),
///     ))
///   )(input)
/// }
///
/// assert_eq!(parser("ab\\\"cd"), Ok(("", String::from("ab\"cd"))));
/// assert_eq!(parser("ab\\ncd"), Ok(("", String::from("ab\ncd"))));
/// ```
#[cfg(feature = "alloc")]
///
/// **WARNING:** Deprecated, replaced with [`winnow::character::escaped_transform`][crate::character::escaped_transform]
#[deprecated(
  since = "8.0.0",
  note = "Replaced with `winnow::character::escaped_transform`"
)]
pub fn escaped_transform<I, Error, F, G, O1, O2, ExtendItem, Output>(
  mut normal: F,
  control_char: char,
  mut transform: G,
) -> impl FnMut(I) -> IResult<I, Output, Error>
where
  I: Input + Offset,
  <I as Input>::Token: crate::input::AsChar,
  I: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O1: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O2: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  F: Parser<I, O1, Error>,
  G: Parser<I, O2, Error>,
  Error: ParseError<I>,
{
  move |input: I| escaped_transform_internal(input, &mut normal, control_char, &mut transform)
}

#[cfg(feature = "alloc")]
pub(crate) fn escaped_transform_internal<I, Error, F, G, O1, O2, ExtendItem, Output>(
  input: I,
  normal: &mut F,
  control_char: char,
  transform: &mut G,
) -> IResult<I, Output, Error>
where
  I: Input + Offset,
  <I as Input>::Token: crate::input::AsChar,
  I: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O1: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  O2: crate::input::ExtendInto<Item = ExtendItem, Extender = Output>,
  F: Parser<I, O1, Error>,
  G: Parser<I, O2, Error>,
  Error: ParseError<I>,
{
  use crate::input::AsChar;

  let mut offset = 0;
  let mut res = input.new_builder();

  let i = input.clone();

  while offset < i.input_len() {
    let current_len = i.input_len();
    let (remainder, _) = i.next_slice(offset);
    match normal.parse_next(remainder.clone()) {
      Ok((i2, o)) => {
        o.extend_into(&mut res);
        if i2.input_len() == 0 {
          return Ok((i.next_slice(i.input_len()).0, res));
        } else if i2.input_len() == current_len {
          return Ok((remainder, res));
        } else {
          offset = input.offset_to(&i2);
        }
      }
      Err(ErrMode::Backtrack(_)) => {
        if remainder.next_token().expect("input_len > 0").1.as_char() == control_char {
          let next = offset + control_char.len_utf8();
          let input_len = input.input_len();

          if next >= input_len {
            return Err(ErrMode::Backtrack(Error::from_error_kind(
              remainder,
              ErrorKind::EscapedTransform,
            )));
          } else {
            match transform.parse_next(i.next_slice(next).0) {
              Ok((i2, o)) => {
                o.extend_into(&mut res);
                if i2.input_len() == 0 {
                  return Ok((i.next_slice(i.input_len()).0, res));
                } else {
                  offset = input.offset_to(&i2);
                }
              }
              Err(e) => return Err(e),
            }
          }
        } else {
          if offset == 0 {
            return Err(ErrMode::Backtrack(Error::from_error_kind(
              remainder,
              ErrorKind::EscapedTransform,
            )));
          }
          return Ok((remainder, res));
        }
      }
      Err(e) => return Err(e),
    }
  }
  Ok((input.next_slice(offset).0, res))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::character::complete::{alpha1 as alpha, digit1 as digit};
  #[cfg(feature = "alloc")]
  use crate::{
    branch::alt,
    combinator::{map, value},
    lib::std::string::String,
    lib::std::vec::Vec,
  };

  #[test]
  fn complete_take_while_m_n_utf8_all_matching() {
    let result: IResult<&str, &str> =
      super::take_while_m_n(1, 4, |c: char| c.is_alphabetic())("øn");
    assert_eq!(result, Ok(("", "øn")));
  }

  #[test]
  fn complete_take_while_m_n_utf8_all_matching_substring() {
    let result: IResult<&str, &str> =
      super::take_while_m_n(1, 1, |c: char| c.is_alphabetic())("øn");
    assert_eq!(result, Ok(("n", "ø")));
  }

  // issue #1336 "escaped hangs if normal parser accepts empty"
  fn escaped_string(input: &str) -> IResult<&str, &str> {
    use crate::character::complete::{alpha0, one_of};
    escaped(alpha0, '\\', one_of("n"))(input)
  }

  // issue #1336 "escaped hangs if normal parser accepts empty"
  #[test]
  fn escaped_hang() {
    escaped_string("7").unwrap();
    escaped_string("a7").unwrap();
  }

  // issue ##1118 escaped does not work with empty string
  fn unquote(input: &str) -> IResult<&str, &str> {
    use crate::bytes::complete::*;
    use crate::character::complete::*;
    use crate::combinator::opt;
    use crate::sequence::delimited;

    delimited(
      char('"'),
      escaped(opt(none_of(r#"\""#)), '\\', one_of(r#"\"rnt"#)),
      char('"'),
    )(input)
  }

  #[test]
  fn escaped_hang_1118() {
    assert_eq!(unquote(r#""""#), Ok(("", "")));
  }

  #[cfg(feature = "alloc")]
  #[allow(unused_variables)]
  #[test]
  fn escaping() {
    use crate::character::complete::one_of;

    fn esc(i: &[u8]) -> IResult<&[u8], &[u8]> {
      escaped(alpha, '\\', one_of("\"n\\"))(i)
    }
    assert_eq!(esc(&b"abcd;"[..]), Ok((&b";"[..], &b"abcd"[..])));
    assert_eq!(esc(&b"ab\\\"cd;"[..]), Ok((&b";"[..], &b"ab\\\"cd"[..])));
    assert_eq!(esc(&b"\\\"abcd;"[..]), Ok((&b";"[..], &b"\\\"abcd"[..])));
    assert_eq!(esc(&b"\\n;"[..]), Ok((&b";"[..], &b"\\n"[..])));
    assert_eq!(esc(&b"ab\\\"12"[..]), Ok((&b"12"[..], &b"ab\\\""[..])));
    assert_eq!(
      esc(&b"AB\\"[..]),
      Err(ErrMode::Backtrack(error_position!(
        &b"AB\\"[..],
        ErrorKind::Escaped
      )))
    );
    assert_eq!(
      esc(&b"AB\\A"[..]),
      Err(ErrMode::Backtrack(error_node_position!(
        &b"AB\\A"[..],
        ErrorKind::Escaped,
        error_position!(&b"A"[..], ErrorKind::OneOf)
      )))
    );

    fn esc2(i: &[u8]) -> IResult<&[u8], &[u8]> {
      escaped(digit, '\\', one_of("\"n\\"))(i)
    }
    assert_eq!(esc2(&b"12\\nnn34"[..]), Ok((&b"nn34"[..], &b"12\\n"[..])));
  }

  #[cfg(feature = "alloc")]
  #[test]
  fn escaping_str() {
    use crate::character::complete::one_of;

    fn esc(i: &str) -> IResult<&str, &str> {
      escaped(alpha, '\\', one_of("\"n\\"))(i)
    }
    assert_eq!(esc("abcd;"), Ok((";", "abcd")));
    assert_eq!(esc("ab\\\"cd;"), Ok((";", "ab\\\"cd")));
    assert_eq!(esc("\\\"abcd;"), Ok((";", "\\\"abcd")));
    assert_eq!(esc("\\n;"), Ok((";", "\\n")));
    assert_eq!(esc("ab\\\"12"), Ok(("12", "ab\\\"")));
    assert_eq!(
      esc("AB\\"),
      Err(ErrMode::Backtrack(error_position!(
        "AB\\",
        ErrorKind::Escaped
      )))
    );
    assert_eq!(
      esc("AB\\A"),
      Err(ErrMode::Backtrack(error_node_position!(
        "AB\\A",
        ErrorKind::Escaped,
        error_position!("A", ErrorKind::OneOf)
      )))
    );

    fn esc2(i: &str) -> IResult<&str, &str> {
      escaped(digit, '\\', one_of("\"n\\"))(i)
    }
    assert_eq!(esc2("12\\nnn34"), Ok(("nn34", "12\\n")));

    fn esc3(i: &str) -> IResult<&str, &str> {
      escaped(alpha, '\u{241b}', one_of("\"n"))(i)
    }
    assert_eq!(esc3("ab␛ncd;"), Ok((";", "ab␛ncd")));
  }

  #[cfg(feature = "alloc")]
  fn to_s(i: Vec<u8>) -> String {
    String::from_utf8_lossy(&i).into_owned()
  }

  #[cfg(feature = "alloc")]
  #[test]
  fn escape_transform() {
    fn esc(i: &[u8]) -> IResult<&[u8], String> {
      map(
        escaped_transform(
          alpha,
          '\\',
          alt((
            value(&b"\\"[..], tag("\\")),
            value(&b"\""[..], tag("\"")),
            value(&b"\n"[..], tag("n")),
          )),
        ),
        to_s,
      )(i)
    }

    assert_eq!(esc(&b"abcd;"[..]), Ok((&b";"[..], String::from("abcd"))));
    assert_eq!(
      esc(&b"ab\\\"cd;"[..]),
      Ok((&b";"[..], String::from("ab\"cd")))
    );
    assert_eq!(
      esc(&b"\\\"abcd;"[..]),
      Ok((&b";"[..], String::from("\"abcd")))
    );
    assert_eq!(esc(&b"\\n;"[..]), Ok((&b";"[..], String::from("\n"))));
    assert_eq!(
      esc(&b"ab\\\"12"[..]),
      Ok((&b"12"[..], String::from("ab\"")))
    );
    assert_eq!(
      esc(&b"AB\\"[..]),
      Err(ErrMode::Backtrack(error_position!(
        &b"\\"[..],
        ErrorKind::EscapedTransform
      )))
    );
    assert_eq!(
      esc(&b"AB\\A"[..]),
      Err(ErrMode::Backtrack(error_node_position!(
        &b"AB\\A"[..],
        ErrorKind::EscapedTransform,
        error_position!(&b"A"[..], ErrorKind::Tag)
      )))
    );

    fn esc2(i: &[u8]) -> IResult<&[u8], String> {
      map(
        escaped_transform(
          alpha,
          '&',
          alt((
            value("è".as_bytes(), tag("egrave;")),
            value("à".as_bytes(), tag("agrave;")),
          )),
        ),
        to_s,
      )(i)
    }
    assert_eq!(
      esc2(&b"ab&egrave;DEF;"[..]),
      Ok((&b";"[..], String::from("abèDEF")))
    );
    assert_eq!(
      esc2(&b"ab&egrave;D&agrave;EF;"[..]),
      Ok((&b";"[..], String::from("abèDàEF")))
    );
  }

  #[cfg(feature = "std")]
  #[test]
  fn escape_transform_str() {
    fn esc(i: &str) -> IResult<&str, String> {
      escaped_transform(
        alpha,
        '\\',
        alt((
          value("\\", tag("\\")),
          value("\"", tag("\"")),
          value("\n", tag("n")),
        )),
      )(i)
    }

    assert_eq!(esc("abcd;"), Ok((";", String::from("abcd"))));
    assert_eq!(esc("ab\\\"cd;"), Ok((";", String::from("ab\"cd"))));
    assert_eq!(esc("\\\"abcd;"), Ok((";", String::from("\"abcd"))));
    assert_eq!(esc("\\n;"), Ok((";", String::from("\n"))));
    assert_eq!(esc("ab\\\"12"), Ok(("12", String::from("ab\""))));
    assert_eq!(
      esc("AB\\"),
      Err(ErrMode::Backtrack(error_position!(
        "\\",
        ErrorKind::EscapedTransform
      )))
    );
    assert_eq!(
      esc("AB\\A"),
      Err(ErrMode::Backtrack(error_node_position!(
        "AB\\A",
        ErrorKind::EscapedTransform,
        error_position!("A", ErrorKind::Tag)
      )))
    );

    fn esc2(i: &str) -> IResult<&str, String> {
      escaped_transform(
        alpha,
        '&',
        alt((value("è", tag("egrave;")), value("à", tag("agrave;")))),
      )(i)
    }
    assert_eq!(esc2("ab&egrave;DEF;"), Ok((";", String::from("abèDEF"))));
    assert_eq!(
      esc2("ab&egrave;D&agrave;EF;"),
      Ok((";", String::from("abèDàEF")))
    );

    fn esc3(i: &str) -> IResult<&str, String> {
      escaped_transform(
        alpha,
        '␛',
        alt((value("\0", tag("0")), value("\n", tag("n")))),
      )(i)
    }
    assert_eq!(esc3("a␛0bc␛n"), Ok(("", String::from("a\0bc\n"))));
  }
}
