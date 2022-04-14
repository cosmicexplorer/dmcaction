/* Copyright 2022 Danny McClanahan */
/* SPDX-License-Identifier: AGPL-3.0-only */

//! ???

/* Turn all warnings into errors! */
#![deny(warnings)]
/* Warn for missing docs in general, and hard require crate-level docs. */
#![warn(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
/* Make all doctests fail if they produce any warnings. */
#![doc(test(attr(deny(warnings))))]
/* Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be
 * copied and pasted across crates, but there doesn't appear to be a way to include inner attributes
 * from a common source. */
#![deny(
  clippy::all,
  clippy::default_trait_access,
  clippy::expl_impl_clone_on_copy,
  clippy::if_not_else,
  clippy::needless_continue,
  clippy::unseparated_literal_suffix,
  clippy::used_underscore_binding
)]
/* We use inner modules in several places in this crate for ergonomics. */
#![allow(clippy::module_inception)]
/* It is often more clear to show that nothing is being moved. */
#![allow(clippy::match_ref_pats)]
/* Subjective style. */
#![allow(
  clippy::len_without_is_empty,
  clippy::redundant_field_names,
  clippy::too_many_arguments
)]
/* Default isn't as big a deal as people seem to think it is. */
#![allow(clippy::new_without_default, clippy::new_ret_no_self)]
/* Arc<Mutex> can be more clear than needing to grok Orderings: */
#![allow(clippy::mutex_atomic)]

use clap::Parser;
use displaydoc::Display;
use regex::Regex;
use thiserror::Error;

use std::path::PathBuf;

/// Strip a given audio track from a recording containing a copy of that audio.
///
/// This may be especially useful for avoiding copyright infringement claims from online services
/// which operate in the US and perform automated scanning for copyrighted audio in order to conform
/// to the DMCA.
#[derive(Parser, Debug)]
#[clap(author, version)]
struct Cli {
  /// A file which contains PCM-format audio from which the canonical recording should be stripped.
  #[clap(short, long, parse(from_os_str))]
  irl_recording: PathBuf,

  /// A file which contains PCM-format audio that should be stripped from the IRL recording.
  ///
  /// The sample rate need not match that of the IRL recording.
  #[clap(short, long, parse(from_os_str))]
  canonical_recording: PathBuf,

  #[clap(flatten)]
  phase_shift: phase_shift::PhaseShiftParameters,
}

#[derive(Debug, Display, Error)]
enum ParseError {
  /// Failed to parse time in {0} format: {1}
  TimeParseFailure(&'static Regex, String),
  /// Failed to parse integer numeral {0}
  IntegerParseFailure(String),
  /// Invalid crop window {0}: {1}
  InvalidCropWindow(String, String),
}

mod phase_shift {
  use clap::Args;
  use displaydoc::Display;
  use lazy_static::lazy_static;
  use regex::Regex;

  use std::default::Default;

  use super::ParseError;

  #[derive(Args, Debug)]
  pub struct PhaseShiftParameters {
    /// The crop parameters for the IRL recording.
    #[clap(long, default_value_t, parse(try_from_str = time_window))]
    irl_recording_window: RecordingWindow,
    /// The crop parameters for the canonical recording.
    #[clap(long, default_value_t, parse(try_from_str = time_window))]
    canonical_recording_window: RecordingWindow,
  }

  /// {minutes}m{seconds}s
  #[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord)]
  pub struct Duration {
    pub minutes: usize,
    pub seconds: usize,
  }

  fn parse_integer(s: &str) -> Result<usize, ParseError> {
    let result: usize = s
      .parse()
      .map_err(|e| ParseError::IntegerParseFailure(format!("{:?}", e)))?;
    Ok(result)
  }

  fn minutes_and_seconds(s: &str) -> Result<Duration, ParseError> {
    lazy_static! {
      static ref TIME_RE: Regex =
        Regex::new("(?:(?P<minutes>[0-9]+)m)?(?:(?P<seconds>[0-9]+)s)?").unwrap();
    }
    match TIME_RE.captures(s) {
      Some(captures) => {
        let minutes: usize = captures
          .name("minutes")
          .map(|m| parse_integer(m.as_str()))
          .unwrap_or(Ok(0))?;
        let seconds: usize = captures
          .name("seconds")
          .map(|m| parse_integer(m.as_str()))
          .unwrap_or(Ok(0))?;
        Ok(Duration { minutes, seconds })
      }
      None => Err(ParseError::TimeParseFailure(&TIME_RE, s.to_string())),
    }
  }

  #[derive(Debug, Display)]
  pub enum MaybeDuration {
    /// {0}
    Some(Duration),
    /// <no crop>
    None,
  }

  impl Default for MaybeDuration {
    fn default() -> Self {
      Self::None
    }
  }

  impl From<Option<Duration>> for MaybeDuration {
    fn from(value: Option<Duration>) -> Self {
      match value {
        Some(duration) => MaybeDuration::Some(duration),
        None => MaybeDuration::None,
      }
    }
  }

  /// Crop recording to start {start_time} and end {end_time}
  #[derive(Debug, Default, Display)]
  pub struct RecordingWindow {
    /// How far into the clip to crop the beginning of the clip.
    pub start_time: MaybeDuration,
    /// How far into the clip to crop the end of the clip.
    pub end_time: MaybeDuration,
  }

  fn time_window(s: &str) -> Result<RecordingWindow, ParseError> {
    lazy_static! {
      static ref TIME_WINDOW_RE: Regex = Regex::new("(?P<left>[^:]+)?:(?P<right>[^:]+)?").unwrap();
    }
    match TIME_WINDOW_RE.captures(s) {
      Some(captures) => {
        let left: Option<Duration> = captures
          .name("left")
          .map(|m| minutes_and_seconds(m.as_str()))
          .transpose()?;
        let right: Option<Duration> = captures
          .name("right")
          .map(|m| minutes_and_seconds(m.as_str()))
          .transpose()?;
        match (&left, &right) {
          (Some(left), Some(right)) => {
            if left > right {
              return Err(ParseError::InvalidCropWindow(
                s.to_string(),
                "left cannot be greater than right crop point".to_string(),
              ));
            }
          }
          _ => (),
        }
        Ok(RecordingWindow {
          start_time: left.into(),
          end_time: right.into(),
        })
      }
      None => Ok(RecordingWindow {
        start_time: None.into(),
        end_time: None.into(),
      }),
    }
  }
}

fn main() {
  let args = Cli::parse();

  dbg!(args);
}
