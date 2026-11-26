# TypeScript LSP review results

Captured from version 0.1.3 using the bundled example. Paths in the responses are relative to the example workspace. Cases exercise the MCP protocol directly. The rename case checks how the server responds to an unsupported tool.

## definition

Where is the function called on line 2 of main.ts defined?

Tool: go_to_definition

```json
{
  "locations": [
    {
      "character": 17,
      "end_character": 23,
      "end_line": 2,
      "file": "math.ts",
      "line": 2
    }
  ]
}
```

## references

Which lines use double in main.ts? Leave out its definition.

Tool: find_references

```json
{
  "locations": [
    {
      "character": 22,
      "end_character": 28,
      "end_line": 2,
      "file": "main.ts",
      "line": 2
    },
    {
      "character": 30,
      "end_character": 36,
      "end_line": 3,
      "file": "main.ts",
      "line": 3
    }
  ]
}
```

## hover

Show the signature and documentation for double in math.ts.

Tool: hover

```json
{
  "text": "```typescript\nfunction double(count: number): number\n```\nDoubles a count."
}
```

## include-declaration

List references to double, including the definition in math.ts.

Tool: find_references

```json
{
  "locations": [
    {
      "character": 10,
      "end_character": 16,
      "end_line": 1,
      "file": "main.ts",
      "line": 1
    },
    {
      "character": 22,
      "end_character": 28,
      "end_line": 2,
      "file": "main.ts",
      "line": 2
    },
    {
      "character": 30,
      "end_character": 36,
      "end_line": 3,
      "file": "main.ts",
      "line": 3
    },
    {
      "character": 17,
      "end_character": 23,
      "end_line": 2,
      "file": "math.ts",
      "line": 2
    }
  ]
}
```

## utf-16

What does the symbol at main.ts, line 3, UTF-16 character 30 resolve to?

Tool: go_to_definition

```json
{
  "locations": [
    {
      "character": 17,
      "end_character": 23,
      "end_line": 2,
      "file": "math.ts",
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
  "error": "expected a TS/JS source file: notes.txt"
}
```

## unsaved-file

Follow the symbol in my unsaved new.ts buffer using this plugin.

Tool: go_to_definition

```json
{
  "error": "resolving new.ts: No such file or directory (os error 2)"
}
```
