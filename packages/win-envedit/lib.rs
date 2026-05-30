// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Pistonite

#[cfg(windows)]
mod imp_win;
#[cfg(windows)]
pub use imp_win::*;
