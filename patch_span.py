import re

with open("crates/semantic/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace("                                        *span,", "                                        *span,")
content = content.replace("                                        *span,\n                                        DiagnosticCategory::Type,", "                                        span,\n                                        DiagnosticCategory::Type,")

with open("crates/semantic/src/lib.rs", "w") as f:
    f.write(content)
