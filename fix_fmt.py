with open('crates/fmt/src/lib.rs', 'r') as f:
    code = f.read()

code = code.replace('push_str("}")', "push('}')")
code = code.replace('push_str("{")', "push('{')")
code = code.replace('push_str("(")', "push('(')")
code = code.replace('push_str(")")', "push(')')")
code = code.replace('push_str(" ")', "push(' ')")
code = code.replace('push_str("\\n")', "push('\\n')")

with open('crates/fmt/src/lib.rs', 'w') as f:
    f.write(code)
