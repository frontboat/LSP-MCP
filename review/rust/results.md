# Rust LSP review results

Captured from version 0.1.3 using the bundled example. Paths in the responses are relative to the example workspace. Cases exercise the MCP protocol directly. The rename case checks how the server responds to an unsupported tool.

## definition

Where is the function called on line 2 of src/lib.rs defined?

Tool: go_to_definition

```json
{
  "locations": [
    {
      "character": 8,
      "end_character": 14,
      "end_line": 2,
      "file": "src/math.rs",
      "line": 2
    }
  ]
}
```

## references

Which lines use double in src/lib.rs? Leave out its definition.

Tool: find_references

```json
{
  "locations": [
    {
      "character": 31,
      "end_character": 37,
      "end_line": 2,
      "file": "src/lib.rs",
      "line": 2
    },
    {
      "character": 39,
      "end_character": 45,
      "end_line": 3,
      "file": "src/lib.rs",
      "line": 3
    }
  ]
}
```

## hover

Show the signature and documentation for double in src/math.rs.

Tool: hover

```json
{
  "text": "\n```rust\nnavigation_example::math\n```\n\n```rust\npub fn double(count: i32) -> i32\n```\n\n---\n\nDoubles a count."
}
```

## include-declaration

List references to double, including the definition in src/math.rs.

Tool: find_references

```json
{
  "locations": [
    {
      "character": 31,
      "end_character": 37,
      "end_line": 2,
      "file": "src/lib.rs",
      "line": 2
    },
    {
      "character": 39,
      "end_character": 45,
      "end_line": 3,
      "file": "src/lib.rs",
      "line": 3
    },
    {
      "character": 8,
      "end_character": 14,
      "end_line": 2,
      "file": "src/math.rs",
      "line": 2
    }
  ]
}
```

## utf-16

What does the symbol at src/lib.rs, line 3, UTF-16 character 39 resolve to?

Tool: go_to_definition

```json
{
  "locations": [
    {
      "character": 8,
      "end_character": 14,
      "end_line": 2,
      "file": "src/math.rs",
      "line": 2
    }
  ]
}
```

## rename

Using only this plugin, rename double to twice in every file.

Tool: rename

```json
{
  "error": "tool not found"
}
```

## unsupported-file

What is the inferred type of double in notes.txt?

Tool: hover

```json
{
  "error": "expected a Rust source file: notes.txt"
}
```

## unsaved-file

Follow the symbol in my unsaved new.rs buffer using this plugin.

Tool: go_to_definition

```json
{
  "error": "resolving new.rs: No such file or directory (os error 2)"
}
```
