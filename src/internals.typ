#let xodec = plugin("plugin.wasm")

#let codex-version = version(0, 2, 0)

#let get-names(value) = {
  assert.eq(type(value), str)
  let result = xodec.get_names(bytes(value))
  array(result).split(0).map(name => str(bytes(name)))
}
