## 0.3.0

### **BREAKING CHANGES**

- `RouterExt::with_bundled_css`: now only takes the endpoint and main.css path as parameters.
  see `config::set_style_options` and `config::set_style_targets`
- `style::CARTERS_PARSER_OPTIONS` and `style::CARTERS_TARGET_OPTIONS` ⟶ (removed, now applied by default)
- `set_template_dir` ⟶ `config::set_template_dir`

### Improving configurability

When building a real-world app with HYRO myself, I relized that it wasn't very configurable. This release
focuses on the `hyro::config` module, which provides simple APIs for... configuration!

- The template environment can be configured with `config::modify_template_env`. Now you can add custom
  functions, filters, global variables, etc. to your templates.
- Templates' file extensions can be configured with `config::set_template_file_extention`. Changing the
  extension won't change the behavior of the template rendering engine. Helpful if you like the shorter
  `.html.j2` that still works with intellisense.
- Migrated existing configurables like template directory, style options, and style targets to the new API.

## HMR Fixes

- Fixed HMR not triggering for the styles directory if it wasn't configured as a child of the templates directory.
- Fixed HMR rarely dropping form data. Some browsers wouldn't upgrade the HMR's websocket until a message was
  recieved due to non-deterministic browser caching. Fixed by a migration to `parking_lot::Mutex`, maintaining
  a sustained lock during rendering, and restricting HMR's access to global form data.
- Fixed HMR halting rendering when encountering a template parsing error. This is standard behavior during
  release builds since invalid templates is inexcusable, but was changed to skip rendering until the template
  is valid during HMR. Utilizes the optional `unstable_machinery` feature of jinja2 to efficiently test for valid templates.
- Fixed HMR garbling forms for out-of-order elements when re-rendering. Previously assumed that all elements were
  rendered in ascending order.

## Minor changes

- Added an optional second argument to the `module` template function that allows passing form data via. a map.
- Added the `reexports` module, currently re-exporting minijinja and lightningcss
- Colorized output from HMR messages, app errors, and the `bind` function.

## 0.2.1

### Minor changes

- Fixed an issue where changes to an `index` template wouldn't perform a full-reload and would render
  to an incorrect and unused endpoint.

## 0.2.0

### Unreasonable speed improvement for plain templates

When setting up benchmarks, I noticed that large templates without any logic (essentially plain HTML)
took longer to render than a small template with only two inputs. Since, without HMR, templates aren't
modified during runtime, "rendering" plain templates will share a `&'static str`. We don't make any
calls to minijinja or clone data. The only API difference is that endpoints previously returning a
`Html<String>` should now return a `Html<Cow<'static, str>>`.

| Version   | Benchmark        | Average Speed        |
| --------- | ---------------- | -------------------- |
| 0.1.0     | form_and_context | 2.57µs (389K /s)     |
| 0.1.0     | plain            | 7.57µs (132K /s)     |
| **0.2.0** | form_and_context | **2.39µs (418K /s)** |
| **0.2.0** | plain            | **39ns (25.6M /s)**  |

### Minor changes

- Added `set_template_dir` for... setting the template directory. Defaults to `templates`.
- Fixed an issue where `cargo check --release` would fail because of a missing generic at `src/template.rs:57`.

## 0.1.0

Initial commit
