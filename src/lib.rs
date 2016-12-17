// Copyright 2015-2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! encoding_rs is a Gecko-oriented Free Software / Open Source implementation
//! of the [Encoding Standard](https://encoding.spec.whatwg.org/) in Rust.
//! Gecko-oriented means that converting to and from UTF-16 is supported in
//! addition to converting to and from UTF-8, that the performance and
//! streamability goals are browser-oriented and that FFI-friendliness is a
//! goal.
//!
//! # Availability
//!
//! The code is available under the
//! [Apache license, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
//! or the [MIT license](https://opensource.org/licenses/MIT), at your option.
//! See the
//! [`COPYRIGHT`](https://github.com/hsivonen/encoding_rs/blob/master/COPYRIGHT)
//! file for details.
//! The [repository is on GitHub](https://github.com/hsivonen/encoding_rs). The
//! [crate is available on crates.io](https://crates.io/crates/encoding_rs).
//!
//! # Examples
//!
//! Example programs:
//!
//! * [Rust](https://github.com/hsivonen/recode_rs)
//! * [C](https://github.com/hsivonen/recode_c)
//! * [C++](https://github.com/hsivonen/recode_cpp)
//!
//! Decode using the non-streaming API:
//!
//! ```
//! use encoding_rs::*;
//!
//! let expectation = "\u{30CF}\u{30ED}\u{30FC}\u{30FB}\u{30EF}\u{30FC}\u{30EB}\u{30C9}";
//! let bytes = b"\x83n\x83\x8D\x81[\x81E\x83\x8F\x81[\x83\x8B\x83h";
//!
//! let (cow, encoding_used, had_errors) = SHIFT_JIS.decode(bytes);
//! assert_eq!(&cow[..], expectation);
//! assert_eq!(encoding_used, SHIFT_JIS);
//! assert!(!had_errors);
//! ```
//!
//! Decode using the streaming API with minimal `unsafe`:
//!
//! ```
//! use encoding_rs::*;
//!
//! let expectation = "\u{30CF}\u{30ED}\u{30FC}\u{30FB}\u{30EF}\u{30FC}\u{30EB}\u{30C9}";
//!
//! // Use an array of byte slices to demonstrate content arriving piece by
//! // piece from the network.
//! let bytes: [&'static [u8]; 4] = [b"\x83",
//!                                  b"n\x83\x8D\x81",
//!                                  b"[\x81E\x83\x8F\x81[\x83",
//!                                  b"\x8B\x83h"];
//!
//! // Very short output buffer to demonstrate the output buffer getting full.
//! // Normally, you'd use something like `[0u8; 2048]`.
//! let mut buffer_bytes = [0u8; 8];
//! // Rust doesn't allow us to stack-allocate a `mut str` without `unsafe`.
//! let mut buffer: &mut str = unsafe {
//!     std::mem::transmute(&mut buffer_bytes[..])
//! };
//!
//! // How many bytes in the buffer currently hold significant data.
//! let mut bytes_in_buffer = 0usize;
//!
//! // Collect the output to a string for demonstration purposes.
//! let mut output = String::new();
//!
//! // The `Decoder`
//! let mut decoder = SHIFT_JIS.new_decoder();
//!
//! // Track whether we see errors.
//! let mut total_had_errors = false;
//!
//! // Decode using a fixed-size intermediate buffer (for demonstrating the
//! // use of a fixed-size buffer; normally when the output of an incremental
//! // decode goes to a `String` one would use `Decoder.decode_to_string()` to
//! // avoid the intermediate buffer).
//! for input in &bytes[..] {
//!     // The number of bytes already read from current `input` in total.
//!     let mut total_read_from_current_input = 0usize;
//!
//!     loop {
//!         let (result, read, written, had_errors) =
//!             decoder.decode_to_str(&input[total_read_from_current_input..],
//!                                   &mut buffer[bytes_in_buffer..],
//!                                   false);
//!         total_read_from_current_input += read;
//!         bytes_in_buffer += written;
//!         if had_errors {
//!             total_had_errors = true;
//!         }
//!         match result {
//!             CoderResult::InputEmpty => {
//!                 // We have consumed the current input buffer. Break out of
//!                 // the inner loop to get the next input buffer from the
//!                 // outer loop.
//!                 break;
//!             },
//!             CoderResult::OutputFull => {
//!                 // Write the current buffer out and consider the buffer
//!                 // empty.
//!                 output.push_str(&buffer[..bytes_in_buffer]);
//!                 bytes_in_buffer = 0usize;
//!                 continue;
//!             }
//!         }
//!     }
//! }
//!
//! // Process EOF
//! loop {
//!     let (result, _, written, had_errors) =
//!         decoder.decode_to_str(b"",
//!                               &mut buffer[bytes_in_buffer..],
//!                               true);
//!     bytes_in_buffer += written;
//!     if had_errors {
//!         total_had_errors = true;
//!     }
//!     // Write the current buffer out and consider the buffer empty.
//!     // Need to do this here for both `match` arms, because we exit the
//!     // loop on `CoderResult::InputEmpty`.
//!     output.push_str(&buffer[..bytes_in_buffer]);
//!     bytes_in_buffer = 0usize;
//!     match result {
//!         CoderResult::InputEmpty => {
//!             // Done!
//!             break;
//!         },
//!         CoderResult::OutputFull => {
//!             continue;
//!         }
//!     }
//! }
//!
//! assert_eq!(&output[..], expectation);
//! assert!(!total_had_errors);
//! ```
//!
//! ## Web / Browser Focus
//!
//! Both in terms of scope and performance, the focus is on the Web. For scope,
//! this means that encoding_rs implements the Encoding Standard fully and
//! doesn't implement encodings that are not specified in the Encoding
//! Standard. For performance, this means that decoding performance is
//! important as well as performance for encoding into UTF-8 or encoding the
//! Basic Latin range (ASCII) into legacy encodings. Non-Basic Latin needs to
//! be encoded into legacy encodings in only two places in the Web platform: in
//! the query part of URLs, in which case it's a matter of relatively rare
//! error handling, and in form submission, in which case the user action and
//! networking tend to hide the performance of the encoder.
//!
//! Deemphasizing performance of encoding non-Basic Latin text into legacy
//! encodings enables smaller code size thanks to the encoder side using the
//! decode-optimized data tables without having encode-optimized data tables at
//! all. Even in decoders, smaller lookup table size is preferred over avoiding
//! multiplication operations.
//!
//! Additionally, performance is a non-goal for the ASCII-incompatible
//! ISO-2022-JP and UTF-16 encodings, which are rarely used on the Web. For
//! clarity, this means that performance is a non-goal for UTF-16 as used on
//! the wire as an interchange encoding (UTF-16 on the `[u8]` side of the API).
//! Good performance for UTF-16 used as an in-RAM Unicode representation
//! (UTF-16 the `[u16]` side of the API) is a goal.
//!
//! Despite the focus on the Web, encoding_rs may well be useful for decoding
//! email, although you'll need to implement UTF-7 decoding and label handling
//! by other means. (Due to the Web focus, patches to add UTF-7 are unwelcome
//! in encoding_rs itself.) Also, despite the browser focus, the hope is that
//! non-browser applications that wish to consume Web content or submit Web
//! forms in a Web-compatible way will find encoding_rs useful.
//!
//! # Streaming & Non-Streaming; Rust & C/C++
//!
//! The API in Rust has two modes of operation: streaming and non-streaming.
//! The streaming API is the foundation of the implementation and should be
//! used when processing data that arrives piecemeal from an i/o stream. The
//! streaming API has an FFI wrapper that exposes it to C callers. The
//! non-streaming part of the API is for Rust callers only and is implemented
//! on top of the streaming API and, as such, could be considered as merely a
//! set of convenience methods. There is no analogous C API exposed via FFI,
//! mainly because C doesn't have standard types for growable byte buffers and
//! Unicode strings that know their length.
//!
//! The C API (header file generated at `target/include/encoding_rs.h` when
//! building encoding_rs) can, in turn, be wrapped for use from C++. Such a
//! C++ wrapper could re-create the non-streaming API in C++ for C++ callers.
//! Currently, encoding_rs comes with a
//! [C++ wrapper](https://github.com/hsivonen/encoding_rs/blob/master/include/encoding_rs_cpp.h)
//! that uses STL+[GSL](https://github.com/Microsoft/GSL/) types, but this
//! wrapper doesn't provide non-streaming convenience methods at this time. A
//! C++ wrapper with XPCOM/MFBT types is planned but does not exist yet.
//!
//! The `Encoding` type is common to both the streaming and non-streaming
//! modes. In the streaming mode, decoding operations are performed with a
//! `Decoder` and encoding operations with an `Encoder` object obtained via
//! `Encoding`. In the non-streaming mode, decoding and encoding operations are
//! performed using methods on `Encoding` objects themselves, so the `Decoder`
//! and `Encoder` objects are not used at all.
//!
//! # Memory management
//!
//! The non-streaming mode never performs heap allocations (even the methods
//! that write into a `Vec<u8>` or a `String` by taking them as arguments do
//! not reallocate the backing buffer of the `Vec<u8>` or the `String`). That
//! is, the non-streaming mode uses caller-allocated buffers exclusively.
//!
//! The methods of the streaming mode that return a `Vec<u8>` or a `String`
//! perform heap allocations but only to allocate the backing buffer of the
//! `Vec<u8>` or the `String`.
//!
//! `Encoding` is always statically allocated. `Decoder` and `Encoder` need no
//! `Drop` cleanup.
//!
//! # Buffer reading and writing behavior
//!
//! Based on experience gained with the `java.nio.charset` encoding converter
//! API and with the Gecko uconv encoding converter API, the buffer reading
//! and writing behaviors of encoding_rs are asymmetric: input buffers are
//! fully drained but output buffers are not always fully filled.
//!
//! When reading from an input buffer, encoding_rs always consumes all input
//! up to the next error or to the end of the buffer. In particular, when
//! decoding, even if the input buffer ends in the middle of a byte sequence
//! for a character, the decoder consumes all input. This has the benefit that
//! the caller of the API can always fill the next buffer from the start from
//! whatever source the bytes come from and never has to first copy the last
//! bytes of the previous buffer to the start of the next buffer. However, when
//! encoding, the UTF-8 input buffers have to end at a character boundary, which
//! is a requirement for the Rust `str` type anyway, and UTF-16 input buffer
//! boundaries falling in the middle of a surrogate pair result in both
//! suggorates being treated individually as unpaired surrogates.
//!
//! Additionally, decoders guarantee that they can be fed even one byte at a
//! time and encoders guarantee that they can be fed even one code point at a
//! time. This has the benefit of not placing restrictions on the size of
//! chunks the content arrives e.g. from network.
//!
//! When writing into an output buffer, encoding_rs makes sure that the code
//! unit sequence for a character is never split across output buffer
//! boundaries. This may result in wasted space at the end of an output buffer,
//! but the advantages are that the output side of both decoders and encoders
//! is greatly simplified compared to designs that attempt to fill output
//! buffers exactly even when that entails splitting a code unit sequence and
//! when encoding_rs methods return to the caller, the output produces thus
//! far is always valid taken as whole. (In the case of encoding to ISO-2022-JP,
//! the output needs to be considered as a whole, because the latest output
//! buffer taken alone might not be valid taken alone if the transition away
//! from the ASCII state occurred in an earlier output buffer. However, since
//! the ISO-2022-JP decoder doesn't treat streams that don't end in the ASCII
//! state as being in error despite the encoder generating a transition to the
//! ASCII state at the end, the claim about the partial output taken as a whole
//! being valid is true even for ISO-2022-JP.)
//!
//! # Error Reporting
//!
//! Based on experience gained with the `java.nio.charset` encoding converter
//! API and with the Gecko uconv encoding converter API, the error reporting
//! behaviors of encoding_rs are asymmetric: decoder errors include offsets
//! that leave it up to the caller to extract the erroneous bytes from the
//! input stream if the caller wishes to do so but encoder errors provide the
//! code point associated with the error without requiring the caller to
//! extract it from the input on its own.
//!
//! On the encoder side, an error is always triggered by the most recently
//! pushed Unicode scalar, which makes it simple to pass the `char` to the
//! caller. Also, it's very typical for the caller to wish to do something with
//! this data: generate a numeric escape for the character. Additionally, the
//! ISO-2022-JP encoder reports U+FFFD instead of the actual input character in
//! certain cases, so requiring the caller to extract the character from the
//! input buffer would require the caller to handle ISO-2022-JP details.
//! Furthermore, requiring the caller to extract the character from the input
//! buffer would require the caller to implement UTF-8 or UTF-16 math, which is
//! the job of an encoding conversion library.
//!
//! On the decoder side, errors are triggered in more complex ways. For
//! example, when decoding the sequence ESC, '$', _buffer boundary_, 'A' as
//! ISO-2022-JP, the ESC byte is in error, but this is discovered only after
//! the buffer boundary when processing 'A'. Thus, the bytes in error might not
//! be the ones most recently pushed to the decoder and the error might not even
//! be in the current buffer.
//!
//! Some encoding conversion APIs address the problem by not acknowledging
//! trailing bytes of an input buffer as consumed if it's still possible for
//! future bytes to cause the trailing bytes to be in error. This way, error
//! reporting can always refer to the most recently pushed buffer. This has the
//! problem that the caller of the API has to copy the unconsumed trailing
//! bytes to the start of the next buffer before being able to fill the rest
//! of the next buffer. This is annoying, error-prone and inefficient.
//!
//! A possible solution would be making the decoder remember recently consumed
//! bytes in order to be able to include a copy of the erroneous bytes when
//! reporting an error. This has two problem: First, callers a rarely
//! interested in the erroneous bytes, so attempts to identify them are most
//! often just overhead anyway. Second, the rare applications that are
//! interested typically care about the location of the error in the input
//! stream.
//!
//! To keep the API convenient for common uses and the overhead low while making
//! it possible to develop applications, such as HTML validators, that care
//! about which bytes were in error, encoding_rs reports the length of the
//! erroneous sequence and the number of bytes consumed after the erroneous
//! sequence. As long as the caller doesn't discard the 6 most recent bytes,
//! this makes it possible for callers that care about the erroneous bytes to
//! locate them.
//!
//! # No Convenience API for Custom Replacements
//!
//! The Web Platform and, therefore, the Encoding Standard supports only one
//! error recovery mode for decoders and only one error recovery mode for
//! encoders. The supported error recovery mode for decoders is emitting the
//! REPLACEMENT CHARACTER on error. The supported error recovery mode for
//! encoders is emitting an HTML decimal numeric character reference for
//! unmappable characters.
//!
//! Since encoding_rs is Web-focused, these are the only error recovery modes
//! for which convenient support is provided. Moreover, on the decoder side,
//! there aren't really good alternatives for emitting the REPLACEMENT CHARACTER
//! on error (other than treating errors as fatal). In particular, simply
//! ignoring errors is a
//! [security problem](http://www.unicode.org/reports/tr36/#Substituting_for_Ill_Formed_Subsequences),
//! so it would be a bad idea for encoding_rs to provide a mode that encouraged
//! callers to ignore errors.
//!
//! On the encoder side, there are plausible alternatives for HTML decimal
//! numeric character references. For example, when outputting CSS, CSS-style
//! escapes would seem to make sense. However, instead of facilitating the
//! output of CSS, JS, etc. in non-UTF-8 encodings, encoding_rs takes the design
//! position that you shouldn't generate output in encodings other than UTF-8,
//! except where backward compatibility with interacting with the legacy Web
//! requires it. The legacy Web requires it only when parsing the query strings
//! of URLs and when submitting forms, and those two both use HTML decimal
//! numeric character references.
//!
//! While encoding_rs doesn't make encoder replacements other than HTML decimal
//! numeric character references easy, it does make them _possible_.
//! `encode_from_utf8()`, which emits HTML decimal numeric character references
//! for unmappable characters, is implemented on top of
//! `encode_from_utf8_without_replacement()`. Applications that really, really
//! want other replacement schemes for unmappable characters can likewise
//! implement them on top of `encode_from_utf8_without_replacement()`.
//!
//! # No Extensibility by Design
//!
//! The set of encodings supported by encoding_rs is not extensible by design.
//! That is, `Encoding`, `Decoder` and `Encoder` are intentionally `struct`s
//! rather than `trait`s. encoding_rs takes the design position that all future
//! text interchange should be done using UTF-8, which can represent all of
//! Unicode. (It is, in fact, the only encoding supported by the Encoding
//! Standard and encoding_rs that can represent all of Unicode and that has
//! encoder support. UTF-16LE and UTF-16BE don't have encoder support, and
//! gb18030 cannot encode U+E5E5.) The other encodings are supported merely for
//! legacy compatibility and not due to non-UTF-8 encodings having benefits
//! other than being able to consume legacy content.
//!
//! Considering that UTF-8 can represent all of Unicode and is already supported
//! by all Web browsers, introducing a new encoding wouldn't add to the
//! expressiveness but would add to compatibility problems. In that sense,
//! adding new encodings to the Web Platform doesn't make sense, and, in fact,
//! post-UTF-8 attempts at encodings, such as BOCU-1, have been rejected from
//! the Web Platform. On the other hand, the set of legacy encodings that must
//! be supported for a Web browser to be able to be successful is not going to
//! expand. Empirically, the set of encodings specified in the Encoding Standard
//! is already sufficient and the set of legacy encodings won't grow
//! retroactively.
//!
//! Since extensibility doesn't make sense considering the Web focus of
//! encoding_rs and adding encodings to Web clients would be actively harmful,
//! it makes sense to make the set of encodings that encoding_rs supports
//! non-extensible and to take the (admittedly small) benefits arising from
//! that, such as the size of `Decoder` and `Encoder` objects being known ahead
//!  of time, which enables stack allocation thereof.
//!
//! This does have downsides for applications that might want to put encoding_rs
//! to non-Web uses if those non-Web uses involve legacy encodings that aren't
//! needed for Web uses. The needs of such applications should not complicate
//! encoding_rs itself, though. It is up to those applications to provide a
//! framework that delegates the operations with encodings that encoding_rs
//! supports to encoding_rs and operations with other encodings to something
//! else (as opposed to encoding_rs itself providing an extensibility
//! framework).
//!
//! # Panics
//!
//! Methods in encoding_rs can panic if the API is used against the requirements
//! stated in the documentation, if a state that's supposed to be impossible
//! is reached due to an internal bug or on integer overflow. When used
//! according to documentation with buffer sizes that stay below integer
//! overflow, in the absence of internal bugs, encoding_rs does not panic.
//!
//! Panics aren't documented beyond this on individual methods.
//!
//! The FFI code does not deal with unwinding across the FFI boundary.
//! Therefore, when using FFI, encoding_rs must be compiled with panics aborting
//! in order to avoid Undefined Behavior.
//!
//! # At-Risk Parts of the API
//!
//! The foreseeable source of partially backward-incompatible API change is the
//! way the instances of `Encoding` are made available.
//!
//! If Rust changes to allow the entries of `[&'static Encoding; N]` to be
//! initialized with `static`s of type `&'static Encoding`, the non-reference
//! `FOO_INIT` public `Encoding` instances will be removed from the public API.
//!
//! If Rust changes to make the referent of `pub const FOO: &'static Encoding`
//! unique when the constant is used in different crates, the reference-typed
//! `static`s for the encoding instances will be changed from `static` to
//! `const` and the non-reference-typed `_INIT` instances will be removed.
//!
//! # Mapping Spec Concepts onto the API
//!
//! <table>
//! <thead>
//! <tr><th>Spec Concept</th><th>Streaming</th><th>Non-Streaming</th></tr>
//! </thead>
//! <tbody>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#encoding">encoding</a></td><td><code>&amp;'static Encoding</code></td><td><code>&amp;'static Encoding</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8">UTF-8 encoding</a></td><td><code>UTF_8</code></td><td><code>UTF_8</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#concept-encoding-get">get an encoding</a></td><td><code>Encoding::for_label(<var>label</var>)</code></td><td><code>Encoding::for_label(<var>label</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#name">name</a></td><td><code><var>encoding</var>.name()</code></td><td><code><var>encoding</var>.name()</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#get-an-output-encoding">get an output encoding</a></td><td><code><var>encoding</var>.output_encoding()</code></td><td><code><var>encoding</var>.output_encoding()</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#decode">decode</a></td><td><code>let d = <var>encoding</var>.new_decoder();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code><var>encoding</var>.decode(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode">UTF-8 decode</a></td><td><code>let d = UTF_8.new_decoder_with_bom_removal();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code>UTF_8.decode_with_bom_removal(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode-without-bom">UTF-8 decode without BOM</a></td><td><code>let d = UTF_8.new_decoder_without_bom_handling();<br>let res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = d.decode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code>UTF_8.decode_without_bom_handling(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail">UTF-8 decode without BOM or fail</a></td><td><code>let d = UTF_8.new_decoder_without_bom_handling();<br>let res = d.decode_to_<var>*</var>_without_replacement(<var>src</var>, <var>dst</var>, false);<br>// &hellip; (fail if malformed)</br>let last_res = d.decode_to_<var>*</var>_without_replacement(<var>src</var>, <var>dst</var>, true);<br>// (fail if malformed)</code></td><td><code>UTF_8.decode_without_bom_handling_and_without_replacement(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#encode">encode</a></td><td><code>let e = <var>encoding</var>.new_encoder();<br>let res = e.encode_to_<var>*</var>(<var>src</var>, <var>dst</var>, false);<br>// &hellip;</br>let last_res = e.encode_to_<var>*</var>(<var>src</var>, <var>dst</var>, true);</code></td><td><code><var>encoding</var>.encode(<var>src</var>)</code></td></tr>
//! <tr><td><a href="https://encoding.spec.whatwg.org/#utf-8-encode">UTF-8 encode</a></td><td>Use the UTF-8 nature of Rust strings directly:<br><code><var>write</var>(<var>src</var>.as_bytes());<br>// refill src<br><var>write</var>(<var>src</var>.as_bytes());<br>// refill src<br><var>write</var>(<var>src</var>.as_bytes());<br>// &hellip;</code></td><td>Use the UTF-8 nature of Rust strings directly:<br><code><var>src</var>.as_bytes()</code></td></tr>
//! </tbody>
//! </table>
//!
//! # Compatibility with the rust-encoding API
//!
//! The crate
//! [encoding_rs_compat](https://github.com/hsivonen/encoding_rs_compat/)
//! is a drop-in replacement for rust-encoding 0.2.32 that implements (most of)
//! the API of rust-encoding 0.2.32 on top of encoding_rs.
//!
//! # Mapping rust-encoding concepts to encoding_rs concepts
//!
//! The following table provides a mapping from rust-encoding constructs to
//! encoding_rs ones.
//!
//! <table>
//! <thead>
//! <tr><th>rust-encoding</th><th>encoding_rs</th></tr>
//! </thead>
//! <tbody>
//! <tr><td><code>encoding::EncodingRef</code></td><td><code>&amp;'static encoding_rs::Encoding</code></td></tr>
//! <tr><td><code>encoding::all::<var>WINDOWS_31J</var></code> (not based on the WHATWG name for some encodings)</td><td><code>encoding_rs::<var>SHIFT_JIS</var></code> (always the WHATWG name uppercased and hyphens replaced with underscores)</td></tr>
//! <tr><td><code>encoding::all::ERROR</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::ASCII</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::ISO_8859_1</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::all::HZ</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::label::encoding_from_whatwg_label(<var>string</var>)</code></td><td><code>encoding_rs::Encoding::for_label(<var>string</var>)</code></td></tr>
//! <tr><td><code><var>enc</var>.whatwg_name()</code> (always lower case)</td><td><code><var>enc</var>.name()</code> (potentially mixed case)</td></tr>
//! <tr><td><code><var>enc</var>.name()</code></td><td>Not available because not in the Encoding Standard</td></tr>
//! <tr><td><code>encoding::decode(<var>bytes</var>, encoding::DecoderTrap::Replace, <var>enc</var>)</code></td><td><code><var>enc</var>.decode(<var>bytes</var>)</code></td></tr>
//! <tr><td><code><var>enc</var>.decode(<var>bytes</var>, encoding::DecoderTrap::Replace)</code></td><td><code><var>enc</var>.decode_without_bom_handling(<var>bytes</var>)</code></td></tr>
//! <tr><td><code><var>enc</var>.encode(<var>string</var>, encoding::EncoderTrap::Replace)</code></td><td><code><var>enc</var>.encode(<var>string</var>)</code></td></tr>
//! <tr><td><code><var>enc</var>.raw_decoder()</code></td><td><code><var>enc</var>.new_decoder_without_bom_handling()</code></td></tr>
//! <tr><td><code><var>enc</var>.raw_encoder()</code></td><td><code><var>enc</var>.new_encoder()</code></td></tr>
//! <tr><td><code>encoding::RawDecoder</code></td><td><code>encoding_rs::Decoder</code></td></tr>
//! <tr><td><code>encoding::RawEncoder</code></td><td><code>encoding_rs::Encoder</code></td></tr>
//! <tr><td><code><var>raw_decoder</var>.raw_feed(<var>src</var>, <var>dst_string</var>)</code></td><td><code><var>dst_string</var>.reserve(<var>decoder</var>.max_utf8_buffer_length_without_replacement(<var>src</var>.len()));<br><var>decoder</var>.decode_to_string_without_replacement(<var>src</var>, <var>dst_string</var>, false)</code></td></tr>
//! <tr><td><code><var>raw_encoder</var>.raw_feed(<var>src</var>, <var>dst_vec</var>)</code></td><td><code><var>dst_vec</var>.reserve(<var>encoder</var>.max_buffer_length_from_utf8_without_replacement(<var>src</var>.len()));<br><var>encoder</var>.encode_from_utf8_to_vec_without_replacement(<var>src</var>, <var>dst_vec</var>, false)</code></td></tr>
//! <tr><td><code><var>raw_decoder</var>.raw_finish(<var>dst</var>)</code></td><td><code><var>dst_string</var>.reserve(<var>decoder</var>.max_utf8_buffer_length_without_replacement(0));<br><var>decoder</var>.decode_to_string_without_replacement(b"", <var>dst</var>, true)</code></td></tr>
//! <tr><td><code><var>raw_encoder</var>.raw_finish(<var>dst</var>)</code></td><td><code><var>dst_vec</var>.reserve(<var>encoder</var>.max_buffer_length_from_utf8_without_replacement(0));<br><var>encoder</var>.encode_from_utf8_to_vec_without_replacement("", <var>dst</var>, true)</code></td></tr>
//! <tr><td><code>encoding::DecoderTrap::Strict</code></td><td><code>decode*</code> methods that have <code>_without_replacement</code> in their name (and treating the `Malformed` result as fatal).</td></tr>
//! <tr><td><code>encoding::DecoderTrap::Replace</code></td><td><code>decode*</code> methods that <i>do not</i> have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::DecoderTrap::Ignore</code></td><td>It is a bad idea to ignore errors due to security issues, but this could be implemented using <code>decode*</code> methods that have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::DecoderTrap::Call(DecoderTrapFunc)</code></td><td>Can be implemented using <code>decode*</code> methods that have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::EncoderTrap::Strict</code></td><td><code>encode*</code> methods that have <code>_without_replacement</code> in their name (and treating the `Unmappable` result as fatal).</td></tr>
//! <tr><td><code>encoding::EncoderTrap::Replace</code></td><td>Can be implemented using <code>encode*</code> methods that have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::EncoderTrap::Ignore</code></td><td>It is a bad idea to ignore errors due to security issues, but this could be implemented using <code>encode*</code> methods that have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::EncoderTrap::NcrEscape</code></td><td><code>encode*</code> methods that <i>do not</i> have <code>_without_replacement</code> in their name.</td></tr>
//! <tr><td><code>encoding::EncoderTrap::Call(EncoderTrapFunc)</code></td><td>Can be implemented using <code>encode*</code> methods that have <code>_without_replacement</code> in their name.</td></tr>
//! </tbody>
//! </table>

#![cfg_attr(feature = "simd-accel", feature(cfg_target_feature, platform_intrinsics))]

#[macro_use]
extern crate cfg_if;

#[cfg(feature = "simd-accel")]
extern crate simd;

#[macro_use]
mod macros;

#[cfg(feature = "simd-accel")]
mod simd_funcs;

#[cfg(test)]
mod testing;

mod single_byte;
mod utf_8;
mod utf_8_core;
mod gb18030;
mod big5;
mod euc_jp;
mod iso_2022_jp;
mod shift_jis;
mod euc_kr;
mod replacement;
mod x_user_defined;
mod utf_16;

mod ascii;
mod handles;
mod data;
mod variant;
pub mod ffi;

use variant::*;
use utf_8::utf8_valid_up_to;
use ascii::ascii_valid_up_to;
pub use ffi::*;

use std::borrow::Cow;

const NCR_EXTRA: usize = 9; // #1114111;

// BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

const LONGEST_LABEL_LENGTH: usize = 19; // cseucpkdfmtjapanese

const LONGEST_NAME_LENGTH: usize = 14; // x-mac-cyrillic

/// The initializer for the Big5 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static BIG5_INIT: Encoding = Encoding {
    name: "Big5",
    variant: VariantEncoding::Big5,
};

/// The Big5 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static BIG5: &'static Encoding = &BIG5_INIT;

/// The initializer for the EUC-JP encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static EUC_JP_INIT: Encoding = Encoding {
    name: "EUC-JP",
    variant: VariantEncoding::EucJp,
};

/// The EUC-JP encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static EUC_JP: &'static Encoding = &EUC_JP_INIT;

/// The initializer for the EUC-KR encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static EUC_KR_INIT: Encoding = Encoding {
    name: "EUC-KR",
    variant: VariantEncoding::EucKr,
};

/// The EUC-KR encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static EUC_KR: &'static Encoding = &EUC_KR_INIT;

/// The initializer for the GBK encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static GBK_INIT: Encoding = Encoding {
    name: "GBK",
    variant: VariantEncoding::Gbk,
};

/// The GBK encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static GBK: &'static Encoding = &GBK_INIT;

/// The initializer for the IBM866 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static IBM866_INIT: Encoding = Encoding {
    name: "IBM866",
    variant: VariantEncoding::SingleByte(data::IBM866_DATA),
};

/// The IBM866 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static IBM866: &'static Encoding = &IBM866_INIT;

/// The initializer for the ISO-2022-JP encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_2022_JP_INIT: Encoding = Encoding {
    name: "ISO-2022-JP",
    variant: VariantEncoding::Iso2022Jp,
};

/// The ISO-2022-JP encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_2022_JP: &'static Encoding = &ISO_2022_JP_INIT;

/// The initializer for the ISO-8859-10 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_10_INIT: Encoding = Encoding {
    name: "ISO-8859-10",
    variant: VariantEncoding::SingleByte(data::ISO_8859_10_DATA),
};

/// The ISO-8859-10 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_10: &'static Encoding = &ISO_8859_10_INIT;

/// The initializer for the ISO-8859-13 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_13_INIT: Encoding = Encoding {
    name: "ISO-8859-13",
    variant: VariantEncoding::SingleByte(data::ISO_8859_13_DATA),
};

/// The ISO-8859-13 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_13: &'static Encoding = &ISO_8859_13_INIT;

/// The initializer for the ISO-8859-14 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_14_INIT: Encoding = Encoding {
    name: "ISO-8859-14",
    variant: VariantEncoding::SingleByte(data::ISO_8859_14_DATA),
};

/// The ISO-8859-14 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_14: &'static Encoding = &ISO_8859_14_INIT;

/// The initializer for the ISO-8859-15 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_15_INIT: Encoding = Encoding {
    name: "ISO-8859-15",
    variant: VariantEncoding::SingleByte(data::ISO_8859_15_DATA),
};

/// The ISO-8859-15 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_15: &'static Encoding = &ISO_8859_15_INIT;

/// The initializer for the ISO-8859-16 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_16_INIT: Encoding = Encoding {
    name: "ISO-8859-16",
    variant: VariantEncoding::SingleByte(data::ISO_8859_16_DATA),
};

/// The ISO-8859-16 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_16: &'static Encoding = &ISO_8859_16_INIT;

/// The initializer for the ISO-8859-2 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_2_INIT: Encoding = Encoding {
    name: "ISO-8859-2",
    variant: VariantEncoding::SingleByte(data::ISO_8859_2_DATA),
};

/// The ISO-8859-2 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_2: &'static Encoding = &ISO_8859_2_INIT;

/// The initializer for the ISO-8859-3 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_3_INIT: Encoding = Encoding {
    name: "ISO-8859-3",
    variant: VariantEncoding::SingleByte(data::ISO_8859_3_DATA),
};

/// The ISO-8859-3 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_3: &'static Encoding = &ISO_8859_3_INIT;

/// The initializer for the ISO-8859-4 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_4_INIT: Encoding = Encoding {
    name: "ISO-8859-4",
    variant: VariantEncoding::SingleByte(data::ISO_8859_4_DATA),
};

/// The ISO-8859-4 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_4: &'static Encoding = &ISO_8859_4_INIT;

/// The initializer for the ISO-8859-5 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_5_INIT: Encoding = Encoding {
    name: "ISO-8859-5",
    variant: VariantEncoding::SingleByte(data::ISO_8859_5_DATA),
};

/// The ISO-8859-5 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_5: &'static Encoding = &ISO_8859_5_INIT;

/// The initializer for the ISO-8859-6 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_6_INIT: Encoding = Encoding {
    name: "ISO-8859-6",
    variant: VariantEncoding::SingleByte(data::ISO_8859_6_DATA),
};

/// The ISO-8859-6 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_6: &'static Encoding = &ISO_8859_6_INIT;

/// The initializer for the ISO-8859-7 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_7_INIT: Encoding = Encoding {
    name: "ISO-8859-7",
    variant: VariantEncoding::SingleByte(data::ISO_8859_7_DATA),
};

/// The ISO-8859-7 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_7: &'static Encoding = &ISO_8859_7_INIT;

/// The initializer for the ISO-8859-8 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_8_INIT: Encoding = Encoding {
    name: "ISO-8859-8",
    variant: VariantEncoding::SingleByte(data::ISO_8859_8_DATA),
};

/// The ISO-8859-8 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_8: &'static Encoding = &ISO_8859_8_INIT;

/// The initializer for the ISO-8859-8-I encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static ISO_8859_8_I_INIT: Encoding = Encoding {
    name: "ISO-8859-8-I",
    variant: VariantEncoding::SingleByte(data::ISO_8859_8_DATA),
};

/// The ISO-8859-8-I encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static ISO_8859_8_I: &'static Encoding = &ISO_8859_8_I_INIT;

/// The initializer for the KOI8-R encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static KOI8_R_INIT: Encoding = Encoding {
    name: "KOI8-R",
    variant: VariantEncoding::SingleByte(data::KOI8_R_DATA),
};

/// The KOI8-R encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static KOI8_R: &'static Encoding = &KOI8_R_INIT;

/// The initializer for the KOI8-U encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static KOI8_U_INIT: Encoding = Encoding {
    name: "KOI8-U",
    variant: VariantEncoding::SingleByte(data::KOI8_U_DATA),
};

/// The KOI8-U encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static KOI8_U: &'static Encoding = &KOI8_U_INIT;

/// The initializer for the Shift_JIS encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static SHIFT_JIS_INIT: Encoding = Encoding {
    name: "Shift_JIS",
    variant: VariantEncoding::ShiftJis,
};

/// The Shift_JIS encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static SHIFT_JIS: &'static Encoding = &SHIFT_JIS_INIT;

/// The initializer for the UTF-16BE encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static UTF_16BE_INIT: Encoding = Encoding {
    name: "UTF-16BE",
    variant: VariantEncoding::Utf16Be,
};

/// The UTF-16BE encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static UTF_16BE: &'static Encoding = &UTF_16BE_INIT;

/// The initializer for the UTF-16LE encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static UTF_16LE_INIT: Encoding = Encoding {
    name: "UTF-16LE",
    variant: VariantEncoding::Utf16Le,
};

/// The UTF-16LE encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static UTF_16LE: &'static Encoding = &UTF_16LE_INIT;

/// The initializer for the UTF-8 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static UTF_8_INIT: Encoding = Encoding {
    name: "UTF-8",
    variant: VariantEncoding::Utf8,
};

/// The UTF-8 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static UTF_8: &'static Encoding = &UTF_8_INIT;

/// The initializer for the gb18030 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static GB18030_INIT: Encoding = Encoding {
    name: "gb18030",
    variant: VariantEncoding::Gb18030,
};

/// The gb18030 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static GB18030: &'static Encoding = &GB18030_INIT;

/// The initializer for the macintosh encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static MACINTOSH_INIT: Encoding = Encoding {
    name: "macintosh",
    variant: VariantEncoding::SingleByte(data::MACINTOSH_DATA),
};

/// The macintosh encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static MACINTOSH: &'static Encoding = &MACINTOSH_INIT;

/// The initializer for the replacement encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static REPLACEMENT_INIT: Encoding = Encoding {
    name: "replacement",
    variant: VariantEncoding::Replacement,
};

/// The replacement encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static REPLACEMENT: &'static Encoding = &REPLACEMENT_INIT;

/// The initializer for the windows-1250 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1250_INIT: Encoding = Encoding {
    name: "windows-1250",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1250_DATA),
};

/// The windows-1250 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1250: &'static Encoding = &WINDOWS_1250_INIT;

/// The initializer for the windows-1251 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1251_INIT: Encoding = Encoding {
    name: "windows-1251",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1251_DATA),
};

/// The windows-1251 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1251: &'static Encoding = &WINDOWS_1251_INIT;

/// The initializer for the windows-1252 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1252_INIT: Encoding = Encoding {
    name: "windows-1252",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1252_DATA),
};

/// The windows-1252 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1252: &'static Encoding = &WINDOWS_1252_INIT;

/// The initializer for the windows-1253 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1253_INIT: Encoding = Encoding {
    name: "windows-1253",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1253_DATA),
};

/// The windows-1253 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1253: &'static Encoding = &WINDOWS_1253_INIT;

/// The initializer for the windows-1254 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1254_INIT: Encoding = Encoding {
    name: "windows-1254",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1254_DATA),
};

/// The windows-1254 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1254: &'static Encoding = &WINDOWS_1254_INIT;

/// The initializer for the windows-1255 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1255_INIT: Encoding = Encoding {
    name: "windows-1255",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1255_DATA),
};

/// The windows-1255 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1255: &'static Encoding = &WINDOWS_1255_INIT;

/// The initializer for the windows-1256 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1256_INIT: Encoding = Encoding {
    name: "windows-1256",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1256_DATA),
};

/// The windows-1256 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1256: &'static Encoding = &WINDOWS_1256_INIT;

/// The initializer for the windows-1257 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1257_INIT: Encoding = Encoding {
    name: "windows-1257",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1257_DATA),
};

/// The windows-1257 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1257: &'static Encoding = &WINDOWS_1257_INIT;

/// The initializer for the windows-1258 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_1258_INIT: Encoding = Encoding {
    name: "windows-1258",
    variant: VariantEncoding::SingleByte(data::WINDOWS_1258_DATA),
};

/// The windows-1258 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_1258: &'static Encoding = &WINDOWS_1258_INIT;

/// The initializer for the windows-874 encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static WINDOWS_874_INIT: Encoding = Encoding {
    name: "windows-874",
    variant: VariantEncoding::SingleByte(data::WINDOWS_874_DATA),
};

/// The windows-874 encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static WINDOWS_874: &'static Encoding = &WINDOWS_874_INIT;

/// The initializer for the x-mac-cyrillic encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static X_MAC_CYRILLIC_INIT: Encoding = Encoding {
    name: "x-mac-cyrillic",
    variant: VariantEncoding::SingleByte(data::X_MAC_CYRILLIC_DATA),
};

/// The x-mac-cyrillic encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static X_MAC_CYRILLIC: &'static Encoding = &X_MAC_CYRILLIC_INIT;

/// The initializer for the x-user-defined encoding.
///
/// For use only for taking the address of this form when
/// Rust prohibits the use of the non-`_INIT` form directly,
/// such as in initializers of other `static`s.
///
/// This part of the public API will go away if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate or if Rust starts allowing static arrays
/// to be initialized with `pub static FOO: &'static Encoding`
/// items.
pub static X_USER_DEFINED_INIT: Encoding = Encoding {
    name: "x-user-defined",
    variant: VariantEncoding::UserDefined,
};

/// The x-user-defined encoding.
///
/// This will change from `static` to `const` if Rust changes
/// to make the referent of `pub const FOO: &'static Encoding`
/// unique cross-crate, so don't take the address of this
/// `static`.
pub static X_USER_DEFINED: &'static Encoding = &X_USER_DEFINED_INIT;

static ENCODINGS_SORTED_BY_NAME: [&'static Encoding; 40] = [&BIG5_INIT,
                                                            &EUC_JP_INIT,
                                                            &EUC_KR_INIT,
                                                            &GBK_INIT,
                                                            &IBM866_INIT,
                                                            &ISO_2022_JP_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &ISO_8859_13_INIT,
                                                            &ISO_8859_14_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &ISO_8859_16_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_8_I_INIT,
                                                            &KOI8_R_INIT,
                                                            &KOI8_U_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &UTF_16BE_INIT,
                                                            &UTF_16LE_INIT,
                                                            &UTF_8_INIT,
                                                            &GB18030_INIT,
                                                            &MACINTOSH_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &WINDOWS_1250_INIT,
                                                            &WINDOWS_1251_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &WINDOWS_1253_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1255_INIT,
                                                            &WINDOWS_1256_INIT,
                                                            &WINDOWS_1257_INIT,
                                                            &WINDOWS_1258_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &X_MAC_CYRILLIC_INIT,
                                                            &X_USER_DEFINED_INIT];

static LABELS_SORTED: [&'static str; 218] = ["866",
                                             "ansi_x3.4-1968",
                                             "arabic",
                                             "ascii",
                                             "asmo-708",
                                             "big5",
                                             "big5-hkscs",
                                             "chinese",
                                             "cn-big5",
                                             "cp1250",
                                             "cp1251",
                                             "cp1252",
                                             "cp1253",
                                             "cp1254",
                                             "cp1255",
                                             "cp1256",
                                             "cp1257",
                                             "cp1258",
                                             "cp819",
                                             "cp866",
                                             "csbig5",
                                             "cseuckr",
                                             "cseucpkdfmtjapanese",
                                             "csgb2312",
                                             "csibm866",
                                             "csiso2022jp",
                                             "csiso2022kr",
                                             "csiso58gb231280",
                                             "csiso88596e",
                                             "csiso88596i",
                                             "csiso88598e",
                                             "csiso88598i",
                                             "csisolatin1",
                                             "csisolatin2",
                                             "csisolatin3",
                                             "csisolatin4",
                                             "csisolatin5",
                                             "csisolatin6",
                                             "csisolatin9",
                                             "csisolatinarabic",
                                             "csisolatincyrillic",
                                             "csisolatingreek",
                                             "csisolatinhebrew",
                                             "cskoi8r",
                                             "csksc56011987",
                                             "csmacintosh",
                                             "csshiftjis",
                                             "cyrillic",
                                             "dos-874",
                                             "ecma-114",
                                             "ecma-118",
                                             "elot_928",
                                             "euc-jp",
                                             "euc-kr",
                                             "gb18030",
                                             "gb2312",
                                             "gb_2312",
                                             "gb_2312-80",
                                             "gbk",
                                             "greek",
                                             "greek8",
                                             "hebrew",
                                             "hz-gb-2312",
                                             "ibm819",
                                             "ibm866",
                                             "iso-2022-cn",
                                             "iso-2022-cn-ext",
                                             "iso-2022-jp",
                                             "iso-2022-kr",
                                             "iso-8859-1",
                                             "iso-8859-10",
                                             "iso-8859-11",
                                             "iso-8859-13",
                                             "iso-8859-14",
                                             "iso-8859-15",
                                             "iso-8859-16",
                                             "iso-8859-2",
                                             "iso-8859-3",
                                             "iso-8859-4",
                                             "iso-8859-5",
                                             "iso-8859-6",
                                             "iso-8859-6-e",
                                             "iso-8859-6-i",
                                             "iso-8859-7",
                                             "iso-8859-8",
                                             "iso-8859-8-e",
                                             "iso-8859-8-i",
                                             "iso-8859-9",
                                             "iso-ir-100",
                                             "iso-ir-101",
                                             "iso-ir-109",
                                             "iso-ir-110",
                                             "iso-ir-126",
                                             "iso-ir-127",
                                             "iso-ir-138",
                                             "iso-ir-144",
                                             "iso-ir-148",
                                             "iso-ir-149",
                                             "iso-ir-157",
                                             "iso-ir-58",
                                             "iso8859-1",
                                             "iso8859-10",
                                             "iso8859-11",
                                             "iso8859-13",
                                             "iso8859-14",
                                             "iso8859-15",
                                             "iso8859-2",
                                             "iso8859-3",
                                             "iso8859-4",
                                             "iso8859-5",
                                             "iso8859-6",
                                             "iso8859-7",
                                             "iso8859-8",
                                             "iso8859-9",
                                             "iso88591",
                                             "iso885910",
                                             "iso885911",
                                             "iso885913",
                                             "iso885914",
                                             "iso885915",
                                             "iso88592",
                                             "iso88593",
                                             "iso88594",
                                             "iso88595",
                                             "iso88596",
                                             "iso88597",
                                             "iso88598",
                                             "iso88599",
                                             "iso_8859-1",
                                             "iso_8859-15",
                                             "iso_8859-1:1987",
                                             "iso_8859-2",
                                             "iso_8859-2:1987",
                                             "iso_8859-3",
                                             "iso_8859-3:1988",
                                             "iso_8859-4",
                                             "iso_8859-4:1988",
                                             "iso_8859-5",
                                             "iso_8859-5:1988",
                                             "iso_8859-6",
                                             "iso_8859-6:1987",
                                             "iso_8859-7",
                                             "iso_8859-7:1987",
                                             "iso_8859-8",
                                             "iso_8859-8:1988",
                                             "iso_8859-9",
                                             "iso_8859-9:1989",
                                             "koi",
                                             "koi8",
                                             "koi8-r",
                                             "koi8-ru",
                                             "koi8-u",
                                             "koi8_r",
                                             "korean",
                                             "ks_c_5601-1987",
                                             "ks_c_5601-1989",
                                             "ksc5601",
                                             "ksc_5601",
                                             "l1",
                                             "l2",
                                             "l3",
                                             "l4",
                                             "l5",
                                             "l6",
                                             "l9",
                                             "latin1",
                                             "latin2",
                                             "latin3",
                                             "latin4",
                                             "latin5",
                                             "latin6",
                                             "logical",
                                             "mac",
                                             "macintosh",
                                             "ms932",
                                             "ms_kanji",
                                             "shift-jis",
                                             "shift_jis",
                                             "sjis",
                                             "sun_eu_greek",
                                             "tis-620",
                                             "unicode-1-1-utf-8",
                                             "us-ascii",
                                             "utf-16",
                                             "utf-16be",
                                             "utf-16le",
                                             "utf-8",
                                             "utf8",
                                             "visual",
                                             "windows-1250",
                                             "windows-1251",
                                             "windows-1252",
                                             "windows-1253",
                                             "windows-1254",
                                             "windows-1255",
                                             "windows-1256",
                                             "windows-1257",
                                             "windows-1258",
                                             "windows-31j",
                                             "windows-874",
                                             "windows-949",
                                             "x-cp1250",
                                             "x-cp1251",
                                             "x-cp1252",
                                             "x-cp1253",
                                             "x-cp1254",
                                             "x-cp1255",
                                             "x-cp1256",
                                             "x-cp1257",
                                             "x-cp1258",
                                             "x-euc-jp",
                                             "x-gbk",
                                             "x-mac-cyrillic",
                                             "x-mac-roman",
                                             "x-mac-ukrainian",
                                             "x-sjis",
                                             "x-user-defined",
                                             "x-x-big5"];

static ENCODINGS_IN_LABEL_SORT: [&'static Encoding; 218] = [&IBM866_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &BIG5_INIT,
                                                            &BIG5_INIT,
                                                            &GBK_INIT,
                                                            &BIG5_INIT,
                                                            &WINDOWS_1250_INIT,
                                                            &WINDOWS_1251_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &WINDOWS_1253_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1255_INIT,
                                                            &WINDOWS_1256_INIT,
                                                            &WINDOWS_1257_INIT,
                                                            &WINDOWS_1258_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &IBM866_INIT,
                                                            &BIG5_INIT,
                                                            &EUC_KR_INIT,
                                                            &EUC_JP_INIT,
                                                            &GBK_INIT,
                                                            &IBM866_INIT,
                                                            &ISO_2022_JP_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &GBK_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_8_I_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &KOI8_R_INIT,
                                                            &EUC_KR_INIT,
                                                            &MACINTOSH_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &EUC_JP_INIT,
                                                            &EUC_KR_INIT,
                                                            &GB18030_INIT,
                                                            &GBK_INIT,
                                                            &GBK_INIT,
                                                            &GBK_INIT,
                                                            &GBK_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &IBM866_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &ISO_2022_JP_INIT,
                                                            &REPLACEMENT_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &ISO_8859_13_INIT,
                                                            &ISO_8859_14_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &ISO_8859_16_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_8_I_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &EUC_KR_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &GBK_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &ISO_8859_13_INIT,
                                                            &ISO_8859_14_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &ISO_8859_13_INIT,
                                                            &ISO_8859_14_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_5_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_6_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &KOI8_R_INIT,
                                                            &KOI8_R_INIT,
                                                            &KOI8_R_INIT,
                                                            &KOI8_U_INIT,
                                                            &KOI8_U_INIT,
                                                            &KOI8_R_INIT,
                                                            &EUC_KR_INIT,
                                                            &EUC_KR_INIT,
                                                            &EUC_KR_INIT,
                                                            &EUC_KR_INIT,
                                                            &EUC_KR_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &ISO_8859_15_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &ISO_8859_2_INIT,
                                                            &ISO_8859_3_INIT,
                                                            &ISO_8859_4_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &ISO_8859_10_INIT,
                                                            &ISO_8859_8_I_INIT,
                                                            &MACINTOSH_INIT,
                                                            &MACINTOSH_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &ISO_8859_7_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &UTF_8_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &UTF_16LE_INIT,
                                                            &UTF_16BE_INIT,
                                                            &UTF_16LE_INIT,
                                                            &UTF_8_INIT,
                                                            &UTF_8_INIT,
                                                            &ISO_8859_8_INIT,
                                                            &WINDOWS_1250_INIT,
                                                            &WINDOWS_1251_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &WINDOWS_1253_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1255_INIT,
                                                            &WINDOWS_1256_INIT,
                                                            &WINDOWS_1257_INIT,
                                                            &WINDOWS_1258_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &WINDOWS_874_INIT,
                                                            &EUC_KR_INIT,
                                                            &WINDOWS_1250_INIT,
                                                            &WINDOWS_1251_INIT,
                                                            &WINDOWS_1252_INIT,
                                                            &WINDOWS_1253_INIT,
                                                            &WINDOWS_1254_INIT,
                                                            &WINDOWS_1255_INIT,
                                                            &WINDOWS_1256_INIT,
                                                            &WINDOWS_1257_INIT,
                                                            &WINDOWS_1258_INIT,
                                                            &EUC_JP_INIT,
                                                            &GBK_INIT,
                                                            &X_MAC_CYRILLIC_INIT,
                                                            &MACINTOSH_INIT,
                                                            &X_MAC_CYRILLIC_INIT,
                                                            &SHIFT_JIS_INIT,
                                                            &X_USER_DEFINED_INIT,
                                                            &BIG5_INIT];

// END GENERATED CODE

/// An encoding as defined in the [Encoding Standard][1].
///
/// An _encoding_ defines a mapping from a `u8` sequence to a `char` sequence
/// and, in most cases, vice versa. Each encoding has a name, an output
/// encoding, and one or more labels.
///
/// _Labels_ are ASCII-case-insensitive strings that are used to identify an
/// encoding in formats and protocols. The _name_ of the encoding is the
/// preferred label in the case appropriate for returning from the
/// [`characterSet`][2] property of the `Document` DOM interface, except for
/// the replacement encoding whose name is not one of its labels.
///
/// The _output encoding_ is the encoding used for form submission and URL
/// parsing on Web pages in the encoding. This is UTF-8 for the replacement,
/// UTF-16LE and UTF-16BE encodings and the encoding itself for other
/// encodings.
///
/// [1]: https://encoding.spec.whatwg.org/
/// [2]: https://dom.spec.whatwg.org/#dom-document-characterset
///
/// # Streaming vs. Non-Streaming
///
/// When you have the entire input in a single buffer, you can use the
/// convenience methods [`decode()`][1], [`decode_with_bom_removal()`][2],
/// [`decode_without_bom_handling()`][3],
/// [`decode_without_bom_handling_and_without_replacement()`][4] and
/// [`encode()`][5]. (These methods are available to Rust callers only and are
/// not available in the C API.) Unlike the rest of the API available to Rust,
/// these methods perform heap allocations. You should the `Decoder` and
/// `Encoder` objects when your input is split into multiple buffers or when
/// you want to control the allocation of the output buffers.
///
/// [1]: #method.decode
/// [2]: #method.decode_with_bom_removal
/// [3]: #method.decode_without_bom_handling
/// [4]: #method.decode_without_bom_handling_and_without_replacement
/// [5]: #method.encode
///
/// # Instances
///
/// All instances of `Encoding` are statically allocated and have the `'static`
/// lifetime. There is precisely one unique `Encoding` instance for each
/// encoding defined in the Encoding Standard.
///
/// To obtain a reference to a particular encoding whose identity you know at
/// compile time, use a `static` that refers to enccoding. There is a `static`
/// for each encoding. The `static`s are named in all caps with hyphens
/// replaced with underscores (and in C/C++ have `_ENCODING` appended to the
/// name). For example, if you know at compile time that you will want to
/// decode using the UTF-8 encoding, use the `UTF_8` `static` (`UTF_8_ENCODING`
/// in C/C++).
///
/// Additionally, there are non-reference-typed forms ending with `_INIT` to
/// work around the problem that `static`s of the type `&'static Encoding`
/// cannot be used to initialize items of an array whose type is
/// `[&'static Encoding; N]`.
///
/// If you don't know what encoding you need at compile time and need to
/// dynamically get an encoding by label, use
/// <code>Encoding::<a href="#method.for_label">for_label</a>(<var>label</var>)</code>.
///
/// Instances of `Encoding` can be compared with `==` (in both Rust and in
/// C/C++).
pub struct Encoding {
    name: &'static str,
    variant: VariantEncoding,
}

impl Encoding {
    /// Implements the
    /// [_get an encoding_](https://encoding.spec.whatwg.org/#concept-encoding-get)
    /// algorithm.
    ///
    /// If, after ASCII-lowercasing and removing leading and trailing
    /// whitespace, the argument matches a label defined in the Encoding
    /// Standard, `Some(&'static Encoding)` representing the corresponding
    /// encoding is returned. If there is no match, `None` is returned.
    ///
    /// The argument is of type `&[u8]` instead of `&str` to save callers
    /// that are extracting the label from a non-UTF-8 protocol the trouble
    /// of conversion to UTF-8. (If you have a `&str`, just call `.as_bytes()`
    /// on it.)
    ///
    /// Available via the C wrapper.
    pub fn for_label(label: &[u8]) -> Option<&'static Encoding> {
        let mut trimmed = [0u8; LONGEST_LABEL_LENGTH];
        let mut trimmed_pos = 0usize;
        let mut iter = label.into_iter();
        // before
        loop {
            match iter.next() {
                None => {
                    return None;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            continue;
                        }
                        b'A'...b'Z' => {
                            trimmed[trimmed_pos] = *byte + 0x20u8;
                            trimmed_pos = 1usize;
                            break;
                        }
                        // XXX reject bytes that aren't allowed in labels
                        _ => {
                            trimmed[trimmed_pos] = *byte;
                            trimmed_pos = 1usize;
                            break;
                        }
                    }
                }
            }
        }
        // inside
        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            break;
                        }
                        b'A'...b'Z' => {
                            trimmed[trimmed_pos] = *byte + 0x20u8;
                            trimmed_pos += 1usize;
                            if trimmed_pos == LONGEST_LABEL_LENGTH {
                                // There's no encoding with a label this long
                                return None;
                            }
                            continue;
                        }
                        // XXX reject bytes that aren't allowed in labels
                        _ => {
                            trimmed[trimmed_pos] = *byte;
                            trimmed_pos += 1usize;
                            if trimmed_pos == LONGEST_LABEL_LENGTH {
                                // There's no encoding with a label this long
                                return None;
                            }
                            continue;
                        }
                    }
                }
            }

        }
        // after
        loop {
            match iter.next() {
                None => {
                    break;
                }
                Some(byte) => {
                    match *byte {
                        0x09u8 | 0x0Au8 | 0x0Cu8 | 0x0Du8 | 0x20u8 => {
                            continue;
                        }
                        _ => {
                            // There's no label with space in the middle
                            return None;
                        }
                    }
                }
            }

        }
        let candidate = &trimmed[..trimmed_pos];
        // XXX optimize this to binary search, potentially with a comparator
        // that reads the name from the end to start.
        for i in 0..LABELS_SORTED.len() {
            let l = LABELS_SORTED[i];
            if candidate == l.as_bytes() {
                return Some(ENCODINGS_IN_LABEL_SORT[i]);
            }
        }
        return None;
    }

    /// This method behaves the same as `for_label()`, except when `for_label()`
    /// would return `Some(REPLACEMENT)`, this method returns `None` instead.
    ///
    /// This method is useful in scenarios where a fatal error is required
    /// upon invalid label, because in those cases the caller typically wishes
    /// to treat the labels that map to the replacement encoding as fatal
    /// errors, too.
    ///
    /// Available via the C wrapper.
    pub fn for_label_no_replacement(label: &[u8]) -> Option<&'static Encoding> {
        match Encoding::for_label(label) {
            None => None,
            Some(encoding) => {
                if encoding == REPLACEMENT {
                    None
                } else {
                    Some(encoding)
                }
            }
        }
    }

    /// If the argument matches exactly (case-sensitively; no whitespace
    /// removal performed) the name of an encoding, returns
    /// `Some(&'static Encoding)` representing that encoding. Otherwise,
    /// return `None`.
    ///
    /// The motivating use case for this method is interoperability with
    /// legacy Gecko code that represents encodings as name string instead of
    /// type-safe `Encoding` objects. Using this method for other purposes is
    /// most likely the wrong thing to do.
    ///
    /// XXX: Should this method be made FFI-only to discourage Rust callers?
    ///
    /// Available via the C wrapper.
    pub fn for_name(name: &[u8]) -> Option<&'static Encoding> {
        // XXX optimize this to binary search, potentially with a comparator
        // that reads the name from the end to start.
        for i in 0..ENCODINGS_SORTED_BY_NAME.len() {
            let encoding = ENCODINGS_SORTED_BY_NAME[i];
            if name == encoding.name().as_bytes() {
                return Some(ENCODINGS_IN_LABEL_SORT[i]);
            }
        }
        return None;
    }

    /// Performs non-incremental BOM sniffing.
    ///
    /// The argument must either be a buffer representing the entire input
    /// stream (non-streaming case) or a buffer representing at least the first
    /// three bytes of the input stream (streaming case).
    ///
    /// Returns `Some((UTF_8, 3))`, `Some((UTF_16LE, 2))` or
    /// `Some((UTF_16BE, 3))` if the argument starts with the UTF-8, UTF-16LE
    /// or UTF-16BE BOM or `None` otherwise.
    ///
    /// Available via the C wrapper.
    pub fn for_bom(buffer: &[u8]) -> Option<(&'static Encoding, usize)> {
        if buffer.starts_with(b"\xEF\xBB\xBF") {
            Some((UTF_8, 3))
        } else if buffer.starts_with(b"\xFF\xFE") {
            Some((UTF_16LE, 2))
        } else if buffer.starts_with(b"\xFE\xFF") {
            Some((UTF_16BE, 2))
        } else {
            None
        }
    }

    /// Returns the name of this encoding.
    ///
    /// This name is appropriate to return as-is from the DOM
    /// `document.characterSet` property.
    ///
    /// Available via the C wrapper.
    pub fn name(&'static self) -> &'static str {
        self.name
    }

    /// Checks whether the _output encoding_ of this encoding can encode every
    /// `char`. (Only true if the output encoding is UTF-8.)
    ///
    /// Available via the C wrapper.
    pub fn can_encode_everything(&'static self) -> bool {
        self.output_encoding() == UTF_8
    }

    /// Checks whether the bytes 0x00...0x7F map exclusively to the characters
    /// U+0000...U+007F and vice versa.
    ///
    /// Available via the C wrapper.
    pub fn is_ascii_compatible(&'static self) -> bool {
        if self == REPLACEMENT || self == UTF_16BE || self == UTF_16LE || self == ISO_2022_JP {
            false
        } else {
            true
        }
    }

    /// Returns the _output encoding_ of this encoding. This is UTF-8 for
    /// UTF-16BE, UTF-16LE and replacement and the encoding itself otherwise.
    ///
    /// Available via the C wrapper.
    pub fn output_encoding(&'static self) -> &'static Encoding {
        if self == REPLACEMENT || self == UTF_16BE || self == UTF_16LE {
            UTF_8
        } else {
            self
        }
    }

    fn new_variant_decoder(&'static self) -> VariantDecoder {
        self.variant.new_variant_decoder()
    }

    /// Instantiates a new decoder for this encoding with BOM sniffing enabled.
    ///
    /// BOM sniffing may cause the returned decoder to morph into a decoder
    /// for UTF-8, UTF-16LE or UTF-16BE instead of this encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Sniff)
    }

    /// Instantiates a new decoder for this encoding with BOM removal.
    ///
    /// If the input starts with bytes that are the BOM for this encoding,
    /// those bytes are removed. However, the decoder never morphs into a
    /// decoder for another encoding: A BOM for another encoding is treated as
    /// (potentially malformed) input to the decoding algorithm for this
    /// encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder_with_bom_removal(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Remove)
    }

    /// Instantiates a new decoder for this encoding with BOM handling disabled.
    ///
    /// If the input starts with bytes that look like a BOM, those bytes are
    /// not treated as a BOM. (Hence, the decoder never morphs into a decoder
    /// for another encoding.)
    ///
    /// _Note:_ If the caller has performed BOM sniffing on its own but has not
    /// removed the BOM, the caller should use `new_decoder_with_bom_removal()`
    /// instead of this method to cause the BOM to be removed.
    ///
    /// Available via the C wrapper.
    pub fn new_decoder_without_bom_handling(&'static self) -> Decoder {
        Decoder::new(self, self.new_variant_decoder(), BomHandling::Off)
    }

    /// Instantiates a new encoder for the output encoding of this encoding.
    ///
    /// Available via the C wrapper.
    pub fn new_encoder(&'static self) -> Encoder {
        let enc = self.output_encoding();
        enc.variant.new_encoder(enc)
    }

    /// Decode complete input to `String` _with BOM sniffing_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// This method implements the (non-streaming version of) the
    /// [_decode_](https://encoding.spec.whatwg.org/#decode) spec concept.
    ///
    /// The second item in the returned tuple is the encoding that was actually
    /// used (which may differ from this encoding thanks to BOM sniffing).
    ///
    /// The third item in the returned tuple indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use `new_decoder()`
    /// when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`, except when the input is valid UTF-8, in which case
    /// no heap allocation is performed and the output is borrowed from the
    /// input instead.
    ///
    /// Available to Rust only.
    pub fn decode<'a>(&'static self, bytes: &'a [u8]) -> (Cow<'a, str>, &'static Encoding, bool) {
        let (encoding, without_bom) = match Encoding::for_bom(bytes) {
            Some((encoding, bom_length)) => (encoding, &bytes[bom_length..]),
            None => (self, bytes),
        };
        let (cow, had_errors) = encoding.decode_without_bom_handling(without_bom);
        (cow, encoding, had_errors)
    }

    /// Decode complete input to `String` _with BOM removal_ and with
    /// malformed sequences replaced with the REPLACEMENT CHARACTER when the
    /// entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode_](https://encoding.spec.whatwg.org/#utf-8-decode) spec
    /// concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_with_bom_removal()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`, except when the input is valid UTF-8, in which case
    /// no heap allocation is performed and the output is borrowed from the
    /// input instead.
    ///
    /// Available to Rust only.
    pub fn decode_with_bom_removal<'a>(&'static self, bytes: &'a [u8]) -> (Cow<'a, str>, bool) {
        let without_bom = if self == UTF_8 && bytes.starts_with(b"\xEF\xBB\xBF") {
            &bytes[3..]
        } else if self == UTF_16LE && bytes.starts_with(b"\xFF\xFE") {
            &bytes[2..]
        } else if self == UTF_16BE && bytes.starts_with(b"\xFE\xFF") {
            &bytes[2..]
        } else {
            bytes
        };
        self.decode_without_bom_handling(without_bom)
    }

    /// Decode complete input to `String` _without BOM handling_ and
    /// with malformed sequences replaced with the REPLACEMENT CHARACTER when
    /// the entire input is available as a single buffer (i.e. the end of the
    /// buffer marks the end of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode without BOM_](https://encoding.spec.whatwg.org/#utf-8-decode-without-bom)
    /// spec concept.
    ///
    /// The second item in the returned pair indicates whether there were
    /// malformed sequences (that were replaced with the REPLACEMENT CHARACTER).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_without_bom_handling()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`, except when the input is valid UTF-8, in which case
    /// no heap allocation is performed and the output is borrowed from the
    /// input instead.
    ///
    /// Available to Rust only.
    pub fn decode_without_bom_handling<'a>(&'static self, bytes: &'a [u8]) -> (Cow<'a, str>, bool) {
        let (mut decoder, mut string, input) = if self.is_ascii_compatible() {
            let valid_up_to = if self == UTF_8 {
                utf8_valid_up_to(bytes)
            } else {
                ascii_valid_up_to(bytes)
            };
            if valid_up_to == bytes.len() {
                let str: &str = unsafe { std::mem::transmute(bytes) };
                return (Cow::Borrowed(str), false);
            }
            let decoder = self.new_decoder_without_bom_handling();
            let mut string = String::with_capacity(valid_up_to +
                                                   decoder.max_utf8_buffer_length(bytes.len() -
                                                                                  valid_up_to));
            unsafe {
                let mut vec = string.as_mut_vec();
                vec.set_len(valid_up_to);
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), vec.as_mut_ptr(), valid_up_to);
            }
            (decoder, string, &bytes[valid_up_to..])
        } else {
            let decoder = self.new_decoder_without_bom_handling();
            let string = String::with_capacity(decoder.max_utf8_buffer_length(bytes.len()));
            (decoder, string, bytes)
        };
        let (result, read, had_errors) = decoder.decode_to_string(input, &mut string, true);
        match result {
            CoderResult::InputEmpty => {
                debug_assert_eq!(read, input.len());
                (Cow::Owned(string), had_errors)
            }
            CoderResult::OutputFull => unreachable!(),
        }
    }

    /// Decode complete input to `String` _without BOM handling_ and
    /// _with malformed sequences treated as fatal_ when the entire input is
    /// available as a single buffer (i.e. the end of the buffer marks the end
    /// of the stream).
    ///
    /// When invoked on `UTF_8`, this method implements the (non-streaming
    /// version of) the
    /// [_UTF-8 decode without BOM or fail_](https://encoding.spec.whatwg.org/#utf-8-decode-without-bom-or-fail)
    /// spec concept.
    ///
    /// Returns `None` if a malformed sequence was encountered and the result
    /// of the decode as `Some(String)` otherwise.
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use
    /// `new_decoder_without_bom_handling()` when decoding segmented input.
    ///
    /// This method performs a single heap allocation for the backing buffer
    /// of the `String`, except when the input is valid UTF-8, in which case
    /// no heap allocation is performed and the output is borrowed from the
    /// input instead.
    ///
    /// Available to Rust only.
    pub fn decode_without_bom_handling_and_without_replacement<'a>(&'static self,
                                                                   bytes: &'a [u8])
                                                                   -> Option<Cow<'a, str>> {
        if self == UTF_8 {
            let valid_up_to = utf8_valid_up_to(bytes);
            if valid_up_to == bytes.len() {
                let str: &str = unsafe { std::mem::transmute(bytes) };
                return Some(Cow::Borrowed(str));
            }
            return None;
        }
        let (mut decoder, mut string, input) = if self.is_ascii_compatible() {
            let valid_up_to = ascii_valid_up_to(bytes);
            if valid_up_to == bytes.len() {
                let str: &str = unsafe { std::mem::transmute(bytes) };
                return Some(Cow::Borrowed(str));
            }
            let decoder = self.new_decoder_without_bom_handling();
            let mut string = String::with_capacity(valid_up_to +
                                                   decoder.max_utf8_buffer_length(bytes.len() -
                                                                                  valid_up_to));
            unsafe {
                let mut vec = string.as_mut_vec();
                vec.set_len(valid_up_to);
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), vec.as_mut_ptr(), valid_up_to);
            }
            (decoder, string, &bytes[valid_up_to..])
        } else {
            let decoder = self.new_decoder_without_bom_handling();
            let string = String::with_capacity(decoder.max_utf8_buffer_length_without_replacement(bytes.len()));
            (decoder, string, bytes)
        };
        let (result, read) = decoder.decode_to_string_without_replacement(input, &mut string, true);
        match result {
            DecoderResult::InputEmpty => {
                debug_assert_eq!(read, input.len());
                Some(Cow::Owned(string))
            }
            DecoderResult::Malformed(_, _) => None,
            DecoderResult::OutputFull => unreachable!(),
        }
    }

    /// Encode complete input to `Vec<u8>` with unmappable characters
    /// replaced with decimal numeric character references when the entire input
    /// is available as a single buffer (i.e. the end of the buffer marks the
    /// end of the stream).
    ///
    /// This method implements the (non-streaming version of) the
    /// [_encode_](https://encoding.spec.whatwg.org/#encode) spec concept. For
    /// the [_UTF-8 encode_](https://encoding.spec.whatwg.org/#utf-8-encode)
    /// spec concept, use <code><var>string</var>.as_bytes()</code> instead of
    /// invoking this method on `UTF_8`.
    ///
    /// The second item in the returned tuple is the encoding that was actually
    /// used (which may differ from this encoding thanks to some encodings
    /// having UTF-8 as their output encoding).
    ///
    /// The third item in the returned tuple indicates whether there were
    /// unmappable characters (that were replaced with HTML numeric character
    /// references).
    ///
    /// _Note:_ It is wrong to use this when the input buffer represents only
    /// a segment of the input instead of the whole input. Use `new_encoder()`
    /// when encoding segmented output.
    ///
    /// When encoding to UTF-8, this method returns a borrow of the input
    /// without a heap allocation. When encoding to something other than UTF-8,
    /// this method performs a single heap allocation for the backing buffer
    /// of the `Vec<u8>` if there are no unmappable characters and potentially
    /// multiple heap allocations if there are. These allocations are tuned
    /// for jemalloc and may not be optimal when using a different allocator
    /// that doesn't use power-of-two buckets.
    ///
    /// Available to Rust only.
    pub fn encode<'a>(&'static self, string: &'a str) -> (Cow<'a, [u8]>, &'static Encoding, bool) {
        let output_encoding = self.output_encoding();
        if output_encoding == UTF_8 {
            return (Cow::Borrowed(string.as_bytes()), output_encoding, false);
        }
        let (mut encoder, mut vec, mut total_read) = if self.is_ascii_compatible() {
            let bytes = string.as_bytes();
            let valid_up_to = ascii_valid_up_to(bytes);
            if valid_up_to == bytes.len() {
                return (Cow::Borrowed(bytes), output_encoding, false);
            }
            let encoder = output_encoding.new_encoder();
            let mut vec: Vec<u8> = Vec::with_capacity((valid_up_to +
                                                       encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len() - valid_up_to))
                                                       .next_power_of_two());
            unsafe {
                vec.set_len(valid_up_to);
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), vec.as_mut_ptr(), valid_up_to);
            }
            (encoder, vec, valid_up_to)
        } else {
            let encoder = output_encoding.new_encoder();
            let vec: Vec<u8> =
            Vec::with_capacity(encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len())
                                      .next_power_of_two());
            (encoder, vec, 0usize)
        };
        let mut total_had_errors = false;
        loop {
            let (result, read, had_errors) = encoder.encode_from_utf8_to_vec(&string[total_read..],
                                                                             &mut vec,
                                                                             true);
            total_read += read;
            if had_errors {
                total_had_errors = true;
            }
            match result {
                CoderResult::InputEmpty => {
                    debug_assert_eq!(total_read, string.len());
                    return (Cow::Owned(vec), output_encoding, total_had_errors);
                }
                CoderResult::OutputFull => {
                    // reserve_exact wants to know how much more on top of current
                    // length--not current capacity.
                    let needed =
                        encoder.max_buffer_length_from_utf8_if_no_unmappables(string.len() -
                                                                              total_read);
                    let rounded = (vec.capacity() + needed).next_power_of_two();
                    let additional = rounded - vec.len();
                    vec.reserve_exact(additional);
                }
            }
        }
    }
}

impl PartialEq for Encoding {
    fn eq(&self, other: &Encoding) -> bool {
        (self as *const Encoding) == (other as *const Encoding)
    }
}

impl Eq for Encoding {}

impl std::fmt::Debug for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Encoding {{ {} }}", self.name)
    }
}

/// Tracks the life cycle of a decoder from BOM sniffing to conversion to end.
#[derive(PartialEq)]
enum DecoderLifeCycle {
    /// The decoder has seen no input yet.
    AtStart,
    /// The decoder has seen no input yet but expects UTF-8.
    AtUtf8Start,
    /// The decoder has seen no input yet but expects UTF-16BE.
    AtUtf16BeStart,
    /// The decoder has seen no input yet but expects UTF-16LE.
    AtUtf16LeStart,
    /// The decoder has seen EF.
    SeenUtf8First,
    /// The decoder has seen EF, BB.
    SeenUtf8Second,
    /// The decoder has seen FE.
    SeenUtf16BeFirst,
    /// The decoder has seen FF.
    SeenUtf16LeFirst,
    /// Saw EF, BB but not BF, there was a buffer boundary after BB and the
    /// underlying decoder reported EF as an error, so we need to remember to
    /// push BB before the next buffer.
    ConvertingWithPendingBB,
    /// No longer looking for a BOM and EOF not yet seen.
    Converting,
    /// EOF has been seen.
    Finished,
}

/// Communicate the BOM handling mode.
enum BomHandling {
    /// Don't handle the BOM
    Off,
    /// Sniff for UTF-8, UTF-16BE or UTF-16LE BOM
    Sniff,
    /// Remove the BOM only if it's the BOM for this encoding
    Remove,
}

/// Result of a (potentially partial) decode or encode operation with
/// replacement.
#[derive(Debug)]
pub enum CoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// conversion process has completed. Otherwise, the caller should call a
    /// decode or encode method again with more input.
    InputEmpty,

    /// The converter cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the converter.
    OutputFull,
}

/// Result of a (potentially partial) decode operation without replacement.
#[derive(Debug)]
pub enum DecoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// decoding process has completed. Otherwise, the caller should call a
    /// decode method again with more input.
    InputEmpty,

    /// The decoder cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the decoder.
    OutputFull,

    /// The decoder encountered a malformed byte sequence.
    ///
    /// The caller must either treat this as a fatal error or must append one
    /// REPLACEMENT CHARACTER (U+FFFD) to the output and then re-push the
    /// the remaining input to the decoder.
    ///
    /// The first wrapped integer indicates the length of the malformed byte
    /// sequence. The second wrapped integer indicates the number of bytes
    /// that were consumed after the malformed sequence. If the second
    /// integer is zero, the last byte that was consumed is the last byte of
    /// the malformed sequence. Note that the malformed bytes may have been part
    /// of an earlier input buffer.
    ///
    /// The first wrapped integer can have values 1, 2 or 3. The second
    /// wrapped integer can have values 0, 1, 2 or 3. This makes the
    /// worst-case sum of the two 6, and the worst case actually happens with
    /// ISO-2022-JP.
    Malformed(u8, u8), // u8 instead of usize to avoid useless bloat
}

/// A converter that decodes a byte stream into Unicode according to a
/// character encoding in a streaming (incremental) manner.
///
/// The various `decode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for
/// both UTF-8 and UTF-16 output buffers.
///
/// A `decode_*` method decodes bytes from `src` into Unicode characters stored
/// into `dst` until one of the following three things happens:
///
/// 1. A malformed byte sequence is encountered (`*_without_replacement`
///    variants only).
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional byte of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input bytes have been processed.
///
/// The `decode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input bytes were read,
/// how many output code units (`u8` when decoding into UTF-8 and `u16`
/// when decoding to UTF-16) were written (except when decoding into `String`,
/// whose length change indicates this), and in the case of the
/// variants performing replacement, a boolean indicating whether an error was
/// replaced with the REPLACEMENT CHARACTER during the call.
///
/// In the case of the `*_without_replacement` variants, the status is a
/// [`DecoderResult`][1] enumeration (possibilities `Malformed`, `OutputFull` and
/// `InputEmpty` corresponding to the three cases listed above).
///
/// In the case of methods whose name does not end with
/// `*_without_replacement`, malformed sequences are automatically replaced
/// with the REPLACEMENT CHARACTER and errors do not cause the methods to
/// return early.
///
/// When decoding to UTF-8, the output buffer must have at least 4 bytes of
/// space. When decoding to UTF-16, the output buffer must have at least two
/// UTF-16 code units (`u16`) of space.
///
/// When decoding to UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the output buffer is at least the length returned by
/// [`max_utf8_buffer_length_without_replacement()`][2]. When decoding to UTF-8
/// with replacement, the length of the output buffer that guarantees the
/// methods not to return indicating that more output space is needed is given
/// by [`max_utf8_buffer_length()`][3]. When decoding to UTF-16 with
/// or without replacement, the length of the output buffer that guarantees
/// the methods not to return indicating that more output space is needed is
/// given by [`max_utf16_buffer_length()`][4].
///
/// The output written into `dst` is guaranteed to be valid UTF-8 or UTF-16,
/// and the output after each `decode_*` call is guaranteed to consist of
/// complete characters. (I.e. the code unit sequence for the last character is
/// guaranteed not to be split across output buffers.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the bytes in `src` have been consumed.
///
/// A `Decoder` object can be used to incrementally decode a byte stream.
///
/// During the processing of a single stream, the caller must call `decode_*`
/// zero or more times with `last` set to `false` and then call `decode_*` at
/// least once with `last` set to `true`. If `decode_*` returns `InputEmpty`,
/// the processing of the stream has ended. Otherwise, the caller must call
/// `decode_*` again with `last` set to `true` (or treat a `Malformed` result as
///  a fatal error).
///
/// Once the stream has ended, the `Decoder` object must not be used anymore.
/// That is, you need to create another one to process another stream.
///
/// When the decoder returns `OutputFull` or the decoder returns `Malformed` and
/// the caller does not wish to treat it as a fatal error, the input buffer
/// `src` may not have been completely consumed. In that case, the caller must
/// pass the unconsumed contents of `src` to `decode_*` again upon the next
/// call.
///
/// [1]: enum.DecoderResult.html 
/// [2]: #method.max_utf8_buffer_length_without_replacement
/// [3]: #method.max_utf8_buffer_length
/// [4]: #method.max_utf16_buffer_length
///
/// # Infinite loops
///
/// When converting with a fixed-size output buffer whose size is too small to
/// accommodate one character of output, an infinite loop ensues. When
/// converting with a fixed-size output buffer, it generally makes sense to
/// make the buffer fairly large (e.g. couple of kilobytes).
pub struct Decoder {
    encoding: &'static Encoding,
    variant: VariantDecoder,
    life_cycle: DecoderLifeCycle,
}

impl Decoder {
    fn new(enc: &'static Encoding, decoder: VariantDecoder, sniffing: BomHandling) -> Decoder {
        Decoder {
            encoding: enc,
            variant: decoder,
            life_cycle: match sniffing {
                BomHandling::Off => DecoderLifeCycle::Converting,
                BomHandling::Sniff => DecoderLifeCycle::AtStart,
                BomHandling::Remove => {
                    if enc == UTF_8 {
                        DecoderLifeCycle::AtUtf8Start
                    } else if enc == UTF_16BE {
                        DecoderLifeCycle::AtUtf16BeStart
                    } else if enc == UTF_16LE {
                        DecoderLifeCycle::AtUtf16LeStart
                    } else {
                        DecoderLifeCycle::Converting
                    }
                }
            },
        }
    }

    /// The `Encoding` this `Decoder` is for.
    ///
    /// BOM sniffing can change the return value of this method during the life
    /// of the decoder.
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// Query the worst-case UTF-16 output size (with or without replacement).
    ///
    /// Returns the size of the output buffer in UTF-16 code units (`u16`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes.
    ///
    /// Since the REPLACEMENT CHARACTER fits into one UTF-16 code unit, the
    /// return value of this method applies also in the
    /// `_without_replacement` case.
    ///
    /// Available via the C wrapper.
    pub fn max_utf16_buffer_length(&self, byte_length: usize) -> usize {
        self.variant.max_utf16_buffer_length(byte_length)
    }

    /// Query the worst-case UTF-8 output size _without replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding without
    /// replacement error handling.
    ///
    /// Note that this value may be too small for the `_with_replacement` case.
    /// Use `max_utf8_buffer_length()` for that case.
    ///
    /// Available via the C wrapper.
    pub fn max_utf8_buffer_length_without_replacement(&self, byte_length: usize) -> usize {
        self.variant.max_utf8_buffer_length_without_replacement(byte_length)
    }

    /// Query the worst-case UTF-8 output size _with replacement_.
    ///
    /// Returns the size of the output buffer in UTF-8 code units (`u8`)
    /// that will not overflow given the current state of the decoder and
    /// `byte_length` number of additional input bytes when decoding with
    /// errors handled by outputting a REPLACEMENT CHARACTER for each malformed
    /// sequence.
    ///
    /// Available via the C wrapper.
    pub fn max_utf8_buffer_length(&self, byte_length: usize) -> usize {
        self.variant.max_utf8_buffer_length(byte_length)
    }

    public_decode_function!(/// Incrementally decode a byte stream into UTF-16
                            /// _without replacement_.
                            ///
                            /// See the documentation of the struct for
                            /// documentation for `decode_*` methods
                            /// collectively.
                            ///
                            /// Available via the C wrapper.
                            ,
                            decode_to_utf16_without_replacement,
                            decode_to_utf16_raw,
                            decode_to_utf16_checking_end,
                            decode_to_utf16_after_one_potential_bom_byte,
                            decode_to_utf16_after_two_potential_bom_bytes,
                            decode_to_utf16_checking_end_with_offset,
                            u16);

    public_decode_function!(/// Incrementally decode a byte stream into UTF-8
                            /// _without replacement_.
                            ///
                            /// See the documentation of the struct for
                            /// documentation for `decode_*` methods
                            /// collectively.
                            ///
                            /// Available via the C wrapper.
                            ,
                            decode_to_utf8_without_replacement,
                            decode_to_utf8_raw,
                            decode_to_utf8_checking_end,
                            decode_to_utf8_after_one_potential_bom_byte,
                            decode_to_utf8_after_two_potential_bom_bytes,
                            decode_to_utf8_checking_end_with_offset,
                            u8);

    /// Incrementally decode a byte stream into UTF-8 with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8` and then zeroes out up to three
    /// bytes that aren't logically part of the write in order to retain the
    /// UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_str_without_replacement(&mut self,
                                             src: &[u8],
                                             dst: &mut str,
                                             last: bool)
                                             -> (DecoderResult, usize, usize) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written) = self.decode_to_utf8_without_replacement(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written)
    }

    /// Incrementally decode a byte stream into UTF-8 using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a pair that contains the `DecoderResult` and the
    /// number of bytes read. The number of bytes written is signaled via
    /// the length of the `String` changing.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_string_without_replacement(&mut self,
                                                src: &[u8],
                                                dst: &mut String,
                                                last: bool)
                                                -> (DecoderResult, usize) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written) =
                self.decode_to_utf8_without_replacement(src, &mut vec[old_len..], last);
            vec.set_len(old_len + written);
            (result, read)
        }
    }

    /// Incrementally decode a byte stream into UTF-16 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn decode_to_utf16(&mut self,
                           src: &[u8],
                           dst: &mut [u16],
                           last: bool)
                           -> (CoderResult, usize, usize, bool) {
        let mut had_errors = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) =
                self.decode_to_utf16_without_replacement(&src[total_read..],
                                                         &mut dst[total_written..],
                                                         last);
            total_read += read;
            total_written += written;
            match result {
                DecoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_errors);
                }
                DecoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_errors);
                }
                DecoderResult::Malformed(_, _) => {
                    had_errors = true;
                    // There should always be space for the U+FFFD, because
                    // otherwise we'd have gotten OutputFull already.
                    dst[total_written] = 0xFFFD;
                    total_written += 1;
                }
            }
        }
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn decode_to_utf8(&mut self,
                          src: &[u8],
                          dst: &mut [u8],
                          last: bool)
                          -> (CoderResult, usize, usize, bool) {
        let mut had_errors = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) =
                self.decode_to_utf8_without_replacement(&src[total_read..],
                                                        &mut dst[total_written..],
                                                        last);
            total_read += read;
            total_written += written;
            match result {
                DecoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_errors);
                }
                DecoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_errors);
                }
                DecoderResult::Malformed(_, _) => {
                    had_errors = true;
                    // There should always be space for the U+FFFD, because
                    // otherwise we'd have gotten OutputFull already.
                    // XXX: is the above comment actually true for UTF-8 itself?
                    // TODO: Consider having fewer bound checks here.
                    dst[total_written] = 0xEFu8;
                    total_written += 1;
                    dst[total_written] = 0xBFu8;
                    total_written += 1;
                    dst[total_written] = 0xBDu8;
                    total_written += 1;
                }
            }
        }
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER with type system signaling
    /// of UTF-8 validity.
    ///
    /// This methods calls `decode_to_utf8` and then zeroes
    /// out up to three bytes that aren't logically part of the write in order
    /// to retain the UTF-8 validity even for the unwritten part of the buffer.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_str(&mut self,
                         src: &[u8],
                         dst: &mut str,
                         last: bool)
                         -> (CoderResult, usize, usize, bool) {
        let bytes: &mut [u8] = unsafe { std::mem::transmute(dst) };
        let (result, read, written, replaced) = self.decode_to_utf8(src, bytes, last);
        let len = bytes.len();
        let mut trail = written;
        while trail < len && ((bytes[trail] & 0xC0) == 0x80) {
            bytes[trail] = 0;
            trail += 1;
        }
        (result, read, written, replaced)
    }

    /// Incrementally decode a byte stream into UTF-8 with malformed sequences
    /// replaced with the REPLACEMENT CHARACTER using a `String` receiver.
    ///
    /// Like the others, this method follows the logic that the output buffer is
    /// caller-allocated. This method treats the capacity of the `String` as
    /// the output limit. That is, this method guarantees not to cause a
    /// reallocation of the backing buffer of `String`.
    ///
    /// The return value is a tuple that contains the `DecoderResult`, the
    /// number of bytes read and a boolean indicating whether replacements
    /// were done. The number of bytes written is signaled via the length of
    /// the `String` changing.
    ///
    /// See the documentation of the struct for documentation for `decode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn decode_to_string(&mut self,
                            src: &[u8],
                            dst: &mut String,
                            last: bool)
                            -> (CoderResult, usize, bool) {
        unsafe {
            let vec = dst.as_mut_vec();
            let old_len = vec.len();
            let capacity = vec.capacity();
            vec.set_len(capacity);
            let (result, read, written, replaced) = self.decode_to_utf8(src,
                                                                        &mut vec[old_len..],
                                                                        last);
            vec.set_len(old_len + written);
            (result, read, replaced)
        }
    }
}

/// Result of a (potentially partial) encode operation without replacement.
#[derive(Debug)]
pub enum EncoderResult {
    /// The input was exhausted.
    ///
    /// If this result was returned from a call where `last` was `true`, the
    /// decoding process has completed. Otherwise, the caller should call a
    /// decode method again with more input.
    InputEmpty,

    /// The encoder cannot produce another unit of output, because the output
    /// buffer does not have enough space left.
    ///
    /// The caller must provide more output space upon the next call and re-push
    /// the remaining input to the decoder.
    OutputFull,

    /// The encoder encountered an unmappable character.
    ///
    /// The caller must either treat this as a fatal error or must append
    /// a placeholder to the output and then re-push the remaining input to the
    /// encoder.
    Unmappable(char),
}

impl EncoderResult {
    fn unmappable_from_bmp(bmp: u16) -> EncoderResult {
        EncoderResult::Unmappable(::std::char::from_u32(bmp as u32).unwrap())
    }
}

/// A converter that encodes a Unicode stream into bytes according to a
/// character encoding in a streaming (incremental) manner.
///
/// The various `encode_*` methods take an input buffer (`src`) and an output
/// buffer `dst` both of which are caller-allocated. There are variants for
/// both UTF-8 and UTF-16 input buffers.
///
/// An `encode_*` method encode characters from `src` into bytes characters
/// stored into `dst` until one of the following three things happens:
///
/// 1. An unmappable character is encountered (`*_without_replacement` variants
///    only).
///
/// 2. The output buffer has been filled so near capacity that the decoder
///    cannot be sure that processing an additional character of input wouldn't
///    cause so much output that the output buffer would overflow.
///
/// 3. All the input characters have been processed.
///
/// The `encode_*` method then returns tuple of a status indicating which one
/// of the three reasons to return happened, how many input code units (`u8`
/// when encoding from UTF-8 and `u16` when encoding from UTF-16) were read,
/// how many output bytes were written (except when encoding into `Vec<u8>`,
/// whose length change indicates this), and in the case of the variants that
/// perform replacement, a boolean indicating whether an unmappable
/// character was replaced with a numeric character reference during the call.
///
/// In the case of the methods whose name ends with
/// `*_without_replacement`, the status is an [`EncoderResult`][1] enumeration
/// (possibilities `Unmappable`, `OutputFull` and `InputEmpty` corresponding to
/// the three cases listed above).
///
/// In the case of methods whose name does not end with
/// `*_without_replacement`, unmappable characters are automatically replaced
/// with the corresponding numeric character references and unmappable
/// characters do not cause the methods to return early.
///
/// When encoding from UTF-8 without replacement, the methods are guaranteed
/// not to return indicating that more output space is needed if the length
/// of the output buffer is at least the length returned by
/// [`max_buffer_length_from_utf8_without_replacement()`][2]. When encoding from
/// UTF-8 with replacement, the length of the output buffer that guarantees the
/// methods not to return indicating that more output space is needed in the
/// absence of unmappable characters is given by
/// [`max_buffer_length_from_utf8_if_no_unmappables()`][3]. When encoding from
/// UTF-16 without replacement, the methods are guaranteed not to return
/// indicating that more output space is needed if the length of the output
/// buffer is at least the length returned by
/// [`max_buffer_length_from_utf16_without_replacement()`][4]. When encoding
/// from UTF-16 with replacement, the the length of the output buffer that
/// guarantees the methods not to return indicating that more output space is
/// needed in the absence of unmappable characters is given by
/// [`max_buffer_length_from_utf16_if_no_unmappables()`][5].
/// When encoding with replacement, applications are not expected to size the
/// buffer for the worst case ahead of time but to resize the buffer if there
/// are unmappable characters. This is why max length queries are only available
/// for the case where there are no unmappable characters.
///
/// When encoding from UTF-8, each `src` buffer _must_ be valid UTF-8. (When
/// calling from Rust, the type system takes care of this.) When encoding from
/// UTF-16, unpaired surrogates in the input are treated as U+FFFD REPLACEMENT
/// CHARACTERS. Therefore, in order for astral characters not to turn into a
/// pair of REPLACEMENT CHARACTERS, the caller must ensure that surrogate pairs
/// are not split across input buffer boundaries.
///
/// After an `encode_*` call returns, the output produced so far, taken as a
/// whole from the start of the stream, is guaranteed to consist of a valid
/// byte sequence in the target encoding. (I.e. the code unit sequence for a
/// character is guaranteed not to be split across output buffers. However, due
/// to the stateful nature of ISO-2022-JP, the stream needs to be considered
/// from the start for it to be valid. For other encodings, the validity holds
/// on a per-output buffer basis.)
///
/// The boolean argument `last` indicates that the end of the stream is reached
/// when all the characters in `src` have been consumed. This argument is needed
/// for ISO-2022-JP and is ignored for other encodings.
///
/// An `Encoder` object can be used to incrementally encode a byte stream.
///
/// During the processing of a single stream, the caller must call `encode_*`
/// zero or more times with `last` set to `false` and then call `encode_*` at
/// least once with `last` set to `true`. If `encode_*` returns `InputEmpty`,
/// the processing of the stream has ended. Otherwise, the caller must call
/// `encode_*` again with `last` set to `true` (or treat an `Unmappable` result
/// as a fatal error).
///
/// Once the stream has ended, the `Encoder` object must not be used anymore.
/// That is, you need to create another one to process another stream.
///
/// When the encoder returns `OutputFull` or the encoder returns `Unmappable`
/// and the caller does not wish to treat it as a fatal error, the input buffer
/// `src` may not have been completely consumed. In that case, the caller must
/// pass the unconsumed contents of `src` to `encode_*` again upon the next
/// call.
///
/// [1]: enum.EncoderResult.html
/// [2]: #method.max_buffer_length_from_utf8_without_replacement
/// [3]: #method.max_buffer_length_from_utf8_if_no_unmappables
/// [4]: #method.max_buffer_length_from_utf16_without_replacement
/// [5]: #method.max_buffer_length_from_utf16_if_no_unmappables
///
/// # Infinite loops
///
/// When converting with a fixed-size output buffer whose size is too small to
/// accommodate one character of output, an infinite loop ensues. When
/// converting with a fixed-size output buffer, it generally makes sense to
/// make the buffer fairly large (e.g. couple of kilobytes).
pub struct Encoder {
    encoding: &'static Encoding,
    variant: VariantEncoder,
}

impl Encoder {
    fn new(enc: &'static Encoding, encoder: VariantEncoder) -> Encoder {
        Encoder {
            encoding: enc,
            variant: encoder,
        }
    }

    /// The `Encoding` this `Encoder` is for.
    pub fn encoding(&self) -> &'static Encoding {
        self.encoding
    }

    /// Query the worst-case output size when encoding from UTF-16 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf16_without_replacement(&self, u16_length: usize) -> usize {
        self.variant.max_buffer_length_from_utf16_without_replacement(u16_length)
    }

    /// Query the worst-case output size when encoding from UTF-8 without
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf8_without_replacement(&self, byte_length: usize) -> usize {
        self.variant.max_buffer_length_from_utf8_without_replacement(byte_length)
    }

    /// Query the worst-case output size when encoding from UTF-16 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `u16_length` number of
    /// additional input code units if there are no unmappable characters in
    /// the input.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf16_if_no_unmappables(&self, u16_length: usize) -> usize {
        self.max_buffer_length_from_utf16_without_replacement(u16_length) +
        if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        }
    }

    /// Query the worst-case output size when encoding from UTF-8 with
    /// replacement.
    ///
    /// Returns the size of the output buffer in bytes that will not overflow
    /// given the current state of the encoder and `byte_length` number of
    /// additional input code units if there are no unmappable characters in
    /// the input.
    ///
    /// Available via the C wrapper.
    pub fn max_buffer_length_from_utf8_if_no_unmappables(&self, byte_length: usize) -> usize {
        self.max_buffer_length_from_utf8_without_replacement(byte_length) +
        if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        }
    }

    /// Incrementally encode into byte stream from UTF-16 _without replacement_.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf16_without_replacement(&mut self,
                                                 src: &[u16],
                                                 dst: &mut [u8],
                                                 last: bool)
                                                 -> (EncoderResult, usize, usize) {
        self.variant.encode_from_utf16_raw(src, dst, last)
    }

    /// Incrementally encode into byte stream from UTF-8 _without replacement_.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf8_without_replacement(&mut self,
                                                src: &str,
                                                dst: &mut [u8],
                                                last: bool)
                                                -> (EncoderResult, usize, usize) {
        self.variant.encode_from_utf8_raw(src, dst, last)
    }

    /// Incrementally encode into byte stream from UTF-8 _without replacement_.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn encode_from_utf8_to_vec_without_replacement(&mut self,
                                                       src: &str,
                                                       dst: &mut Vec<u8>,
                                                       last: bool)
                                                       -> (EncoderResult, usize) {
        unsafe {
            let old_len = dst.len();
            let capacity = dst.capacity();
            dst.set_len(capacity);
            let (result, read, written) =
                self.encode_from_utf8_without_replacement(src, &mut dst[old_len..], last);
            dst.set_len(old_len + written);
            (result, read)
        }
    }

    /// Incrementally encode into byte stream from UTF-16 with unmappable
    /// characters replaced with HTML (decimal) numeric character references.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf16(&mut self,
                             src: &[u16],
                             dst: &mut [u8],
                             last: bool)
                             -> (CoderResult, usize, usize, bool) {
        let effective_dst_len = dst.len() -
                                if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        };
        let mut had_unmappables = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) = self.encode_from_utf16_without_replacement(&src[total_read..],
                                   &mut dst[total_written..effective_dst_len],
                                   last);
            total_read += read;
            total_written += written;
            match result {
                EncoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_unmappables);
                }
                EncoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_unmappables);
                }
                EncoderResult::Unmappable(unmappable) => {
                    had_unmappables = true;
                    debug_assert!(dst.len() - total_written >= NCR_EXTRA + 1);
                    // There are no UTF-16 encoders and even if there were,
                    // they'd never have unmappables.
                    debug_assert!(self.encoding() != UTF_16BE);
                    debug_assert!(self.encoding() != UTF_16LE);
                    // Additionally, Iso2022JpEncoder is responsible for
                    // transitioning to ASCII when returning with Unmappable
                    // from the jis0208 state. That is, when we encode
                    // ISO-2022-JP and come here, the encoder is in either the
                    // ASCII or the Roman state. We are allowed to generate any
                    // printable ASCII excluding \ and ~.
                    total_written += write_ncr(unmappable, &mut dst[total_written..]);
                }
            }
        }
    }

    /// Incrementally encode into byte stream from UTF-8 with unmappable
    /// characters replaced with HTML (decimal) numeric character references.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available via the C wrapper.
    pub fn encode_from_utf8(&mut self,
                            src: &str,
                            dst: &mut [u8],
                            last: bool)
                            -> (CoderResult, usize, usize, bool) {
        let effective_dst_len = dst.len() -
                                if self.encoding().can_encode_everything() {
            0
        } else {
            NCR_EXTRA
        };
        let mut had_unmappables = false;
        let mut total_read = 0usize;
        let mut total_written = 0usize;
        loop {
            let (result, read, written) = self.encode_from_utf8_without_replacement(&src[total_read..],
                                  &mut dst[total_written..effective_dst_len],
                                  last);
            total_read += read;
            total_written += written;
            match result {
                EncoderResult::InputEmpty => {
                    return (CoderResult::InputEmpty, total_read, total_written, had_unmappables);
                }
                EncoderResult::OutputFull => {
                    return (CoderResult::OutputFull, total_read, total_written, had_unmappables);
                }
                EncoderResult::Unmappable(unmappable) => {
                    had_unmappables = true;
                    debug_assert!(dst.len() - total_written >= NCR_EXTRA + 1);
                    debug_assert!(self.encoding() != UTF_16BE);
                    debug_assert!(self.encoding() != UTF_16LE);
                    // Additionally, Iso2022JpEncoder is responsible for
                    // transitioning to ASCII when returning with Unmappable.
                    total_written += write_ncr(unmappable, &mut dst[total_written..]);
                    if total_written >= effective_dst_len {
                        return (CoderResult::OutputFull,
                                total_read,
                                total_written,
                                had_unmappables);
                    }
                }
            }
        }
    }

    /// Incrementally encode into byte stream from UTF-8 with unmappable
    /// characters replaced with HTML (decimal) numeric character references.
    ///
    /// See the documentation of the struct for documentation for `encode_*`
    /// methods collectively.
    ///
    /// Available to Rust only.
    pub fn encode_from_utf8_to_vec(&mut self,
                                   src: &str,
                                   dst: &mut Vec<u8>,
                                   last: bool)
                                   -> (CoderResult, usize, bool) {
        unsafe {
            let old_len = dst.len();
            let capacity = dst.capacity();
            dst.set_len(capacity);
            let (result, read, written, replaced) = self.encode_from_utf8(src,
                                                                          &mut dst[old_len..],
                                                                          last);
            dst.set_len(old_len + written);
            (result, read, replaced)
        }
    }
}

/// Format an unmappable as NCR without heap allocation.
fn write_ncr(unmappable: char, dst: &mut [u8]) -> usize {
    // len is the number of decimal digits needed to represent unmappable plus
    // 3 (the length of "&#" and ";").
    let mut number = unmappable as u32;
    let len = if number >= 1000000u32 {
        10usize
    } else if number >= 100000u32 {
        9usize
    } else if number >= 10000u32 {
        8usize
    } else if number >= 1000u32 {
        7usize
    } else if number >= 100u32 {
        6usize
    } else {
        // Review the outcome of https://github.com/whatwg/encoding/issues/15
        // to see if this case is possible
        5usize
    };
    debug_assert!(number >= 10u32);
    debug_assert!(len <= dst.len());
    let mut pos = len - 1;
    dst[pos] = b';';
    pos -= 1;
    loop {
        let rightmost = number % 10;
        dst[pos] = rightmost as u8 + b'0';
        pos -= 1;
        if number < 10 {
            break;
        }
        number /= 10;
    }
    dst[1] = b'#';
    dst[0] = b'&';
    len
}

// ############## TESTS ###############

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn sniff_to_utf16(initial_encoding: &'static Encoding,
                      expected_encoding: &'static Encoding,
                      bytes: &[u8],
                      expect: &[u16],
                      breaks: &[usize]) {
        let mut decoder = initial_encoding.new_decoder();

        let mut dest: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(bytes.len()));
        let capacity = dest.capacity();
        dest.resize(capacity, 0u16);

        let mut total_written = 0usize;
        let mut start = 0usize;
        for br in breaks {
            let (result, read, written, _) = decoder.decode_to_utf16(&bytes[start..*br],
                                                                     &mut dest[total_written..],
                                                                     false);
            total_written += written;
            assert_eq!(read, *br - start);
            match result {
                CoderResult::InputEmpty => {}
                CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            start = *br;
        }
        let (result, read, written, _) = decoder.decode_to_utf16(&bytes[start..],
                                                                 &mut dest[total_written..],
                                                                 true);
        total_written += written;
        match result {
            CoderResult::InputEmpty => {}
            CoderResult::OutputFull => {
                unreachable!();
            }
        }
        assert_eq!(read, bytes.len() - start);
        assert_eq!(total_written, expect.len());
        assert_eq!(&dest[..total_written], expect);
        assert_eq!(decoder.encoding(), expected_encoding);
    }

    // Any copyright to the test code below this comment is dedicated to the
    // Public Domain. http://creativecommons.org/publicdomain/zero/1.0/

    #[test]
    fn test_bom_sniffing() {
        // ASCII
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[]);
        // UTF-8
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[2]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[4]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[2, 3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 2]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 3]);
        sniff_to_utf16(WINDOWS_1252,
                       UTF_8,
                       b"\xEF\xBB\xBF\x61\x62",
                       &[0x0061u16, 0x0062u16],
                       &[1, 2, 3, 4]);
        sniff_to_utf16(WINDOWS_1252, UTF_8, b"\xEF\xBB\xBF", &[], &[]);
        // Not UTF-8
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB\x61\x62",
                       &[0x00EFu16, 0x00BBu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB\x61\x62",
                       &[0x00EFu16, 0x00BBu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\x61\x62",
                       &[0x00EFu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\x61\x62",
                       &[0x00EFu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB",
                       &[0x00EFu16, 0x00BBu16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xEF\xBB",
                       &[0x00EFu16, 0x00BBu16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xEF", &[0x00EFu16], &[]);
        // Not UTF-16
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFE\x61\x62",
                       &[0x00FEu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFE\x61\x62",
                       &[0x00FEu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xFE", &[0x00FEu16], &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFF\x61\x62",
                       &[0x00FFu16, 0x0061u16, 0x0062u16],
                       &[]);
        sniff_to_utf16(WINDOWS_1252,
                       WINDOWS_1252,
                       b"\xFF\x61\x62",
                       &[0x00FFu16, 0x0061u16, 0x0062u16],
                       &[1]);
        sniff_to_utf16(WINDOWS_1252, WINDOWS_1252, b"\xFF", &[0x00FFu16], &[]);
        // UTF-16
        sniff_to_utf16(WINDOWS_1252, UTF_16BE, b"\xFE\xFF", &[], &[]);
        sniff_to_utf16(WINDOWS_1252, UTF_16BE, b"\xFE\xFF", &[], &[1]);
        sniff_to_utf16(WINDOWS_1252, UTF_16LE, b"\xFF\xFE", &[], &[]);
        sniff_to_utf16(WINDOWS_1252, UTF_16LE, b"\xFF\xFE", &[], &[1]);
    }

    #[test]
    fn test_output_encoding() {
        assert_eq!(REPLACEMENT.output_encoding(), UTF_8);
        assert_eq!(UTF_16BE.output_encoding(), UTF_8);
        assert_eq!(UTF_16LE.output_encoding(), UTF_8);
        assert_eq!(UTF_8.output_encoding(), UTF_8);
        assert_eq!(WINDOWS_1252.output_encoding(), WINDOWS_1252);
        assert_eq!(REPLACEMENT.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_16BE.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_16LE.new_encoder().encoding(), UTF_8);
        assert_eq!(UTF_8.new_encoder().encoding(), UTF_8);
        assert_eq!(WINDOWS_1252.new_encoder().encoding(), WINDOWS_1252);
    }

    #[test]
    fn test_label_resolution() {
        assert_eq!(Encoding::for_label(b"utf-8"), Some(UTF_8));
        assert_eq!(Encoding::for_label(b"UTF-8"), Some(UTF_8));
        assert_eq!(Encoding::for_label(b" \t \n \x0C \n utf-8 \r \n \t \x0C "),
                   Some(UTF_8));
        assert_eq!(Encoding::for_label(b"utf-8 _"), None);
        assert_eq!(Encoding::for_label(b"bogus"), None);
        assert_eq!(Encoding::for_label(b"bogusbogusbogusbogus"), None);
    }

    // XXX generate tests for all labels

    #[test]
    fn test_decode_valid_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.decode(b"abc\x80\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{00E4}");
            }
        }
        assert_eq!(encoding, WINDOWS_1257);
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_invalid_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.decode(b"abc\x80\xA1\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert_eq!(encoding, WINDOWS_1257);
        assert!(had_errors);
    }

    #[test]
    fn test_decode_ascii_only_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.decode(b"abc");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "abc");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert_eq!(encoding, WINDOWS_1257);
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_as_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.decode(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "\u{20AC}\u{00E4}");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert_eq!(encoding, UTF_8);
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_invalid_utf8_as_windows_1257_to_cow() {
        let (cow, encoding, had_errors) =
            WINDOWS_1257.decode(b"\xEF\xBB\xBF\xE2\x82\xAC\x80\xC3\xA4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert_eq!(encoding, UTF_8);
        assert!(had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_as_utf_8_to_cow() {
        let (cow, encoding, had_errors) = UTF_8.decode(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "\u{20AC}\u{00E4}");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert_eq!(encoding, UTF_8);
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_invalid_utf8_as_utf_8_to_cow() {
        let (cow, encoding, had_errors) = UTF_8.decode(b"\xEF\xBB\xBF\xE2\x82\xAC\x80\xC3\xA4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert_eq!(encoding, UTF_8);
        assert!(had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_as_utf_8_to_cow_with_bom_removal() {
        let (cow, had_errors) = UTF_8.decode_with_bom_removal(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "\u{20AC}\u{00E4}");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_as_windows_1257_to_cow_with_bom_removal() {
        let (cow, had_errors) =
            WINDOWS_1257.decode_with_bom_removal(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s,
                           "\u{013C}\u{00BB}\u{00E6}\u{0101}\u{201A}\u{00AC}\u{0106}\u{00A4}");
            }
        }
        assert!(!had_errors);
    }


    #[test]
    fn test_decode_valid_windows_1257_to_cow_with_bom_removal() {
        let (cow, had_errors) = WINDOWS_1257.decode_with_bom_removal(b"abc\x80\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{00E4}");
            }
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_invalid_windows_1257_to_cow_with_bom_removal() {
        let (cow, had_errors) = WINDOWS_1257.decode_with_bom_removal(b"abc\x80\xA1\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert!(had_errors);
    }

    #[test]
    fn test_decode_ascii_only_windows_1257_to_cow_with_bom_removal() {
        let (cow, had_errors) = WINDOWS_1257.decode_with_bom_removal(b"abc");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "abc");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_to_cow_without_bom_handling() {
        let (cow, had_errors) =
            UTF_8.decode_without_bom_handling(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "\u{FEFF}\u{20AC}\u{00E4}");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_invalid_utf8_to_cow_without_bom_handling() {
        let (cow, had_errors) =
            UTF_8.decode_without_bom_handling(b"\xEF\xBB\xBF\xE2\x82\xAC\x80\xC3\xA4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "\u{FEFF}\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert!(had_errors);
    }

    #[test]
    fn test_decode_valid_windows_1257_to_cow_without_bom_handling() {
        let (cow, had_errors) = WINDOWS_1257.decode_without_bom_handling(b"abc\x80\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{00E4}");
            }
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_invalid_windows_1257_to_cow_without_bom_handling() {
        let (cow, had_errors) = WINDOWS_1257.decode_without_bom_handling(b"abc\x80\xA1\xE4");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, "abc\u{20AC}\u{FFFD}\u{00E4}");
            }
        }
        assert!(had_errors);
    }

    #[test]
    fn test_decode_ascii_only_windows_1257_to_cow_without_bom_handling() {
        let (cow, had_errors) = WINDOWS_1257.decode_without_bom_handling(b"abc");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, "abc");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert!(!had_errors);
    }

    #[test]
    fn test_decode_bomful_valid_utf8_to_cow_without_bom_handling_and_without_replacement() {
        match UTF_8.decode_without_bom_handling_and_without_replacement(b"\xEF\xBB\xBF\xE2\x82\xAC\xC3\xA4") {
            Some(cow) => {
               match cow {
                   Cow::Borrowed(s) => {
                       assert_eq!(s, "\u{FEFF}\u{20AC}\u{00E4}");
                   },
                   Cow::Owned(_) => unreachable!(),
               }
            },
            None => unreachable!(),
        }
    }

    #[test]
    fn test_decode_bomful_invalid_utf8_to_cow_without_bom_handling_and_without_replacement() {
        assert!(UTF_8.decode_without_bom_handling_and_without_replacement(b"\xEF\xBB\xBF\xE2\x82\xAC\x80\xC3\xA4").is_none());
    }

    #[test]
    fn test_decode_valid_windows_1257_to_cow_without_bom_handling_and_without_replacement() {
        match WINDOWS_1257.decode_without_bom_handling_and_without_replacement(b"abc\x80\xE4") {
            Some(cow) => {
                match cow {
                    Cow::Borrowed(_) => unreachable!(),
                    Cow::Owned(s) => {
                        assert_eq!(s, "abc\u{20AC}\u{00E4}");
                    }
                }
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn test_decode_invalid_windows_1257_to_cow_without_bom_handling_and_without_replacement() {
        assert!(WINDOWS_1257.decode_without_bom_handling_and_without_replacement(b"abc\x80\xA1\xE4")
                            .is_none());
    }

    #[test]
    fn test_decode_ascii_only_windows_1257_to_cow_without_bom_handling_and_without_replacement() {
        match WINDOWS_1257.decode_without_bom_handling_and_without_replacement(b"abc") {
            Some(cow) => {
                match cow {
                    Cow::Borrowed(s) => {
                        assert_eq!(s, "abc");
                    }
                    Cow::Owned(_) => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn test_encode_ascii_only_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.encode("abc");
        match cow {
            Cow::Borrowed(s) => {
                assert_eq!(s, b"abc");
            }
            Cow::Owned(_) => unreachable!(),
        }
        assert_eq!(encoding, WINDOWS_1257);
        assert!(!had_errors);
    }

    #[test]
    fn test_encode_valid_windows_1257_to_cow() {
        let (cow, encoding, had_errors) = WINDOWS_1257.encode("abc\u{20AC}\u{00E4}");
        match cow {
            Cow::Borrowed(_) => unreachable!(),
            Cow::Owned(s) => {
                assert_eq!(s, b"abc\x80\xE4");
            }
        }
        assert_eq!(encoding, WINDOWS_1257);
        assert!(!had_errors);
    }

}
