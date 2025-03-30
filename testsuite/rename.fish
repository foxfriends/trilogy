#!/usr/bin/env fish
for file in (fd spec.json)
  yj -jt -i < $file > (path change-extension toml $file)
end
