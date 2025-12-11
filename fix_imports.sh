#!/bin/bash

# Fix imports in core files
cd /home/atix/mug/src/core
for f in *.rs; do
  sed -i 's/use crate::branch/use crate::core::branch/g' "$f"
  sed -i 's/use crate::commit/use crate::core::commit/g' "$f"
  sed -i 's/use crate::database/use crate::core::database/g' "$f"
  sed -i 's/use crate::error/use crate::core::error/g' "$f"
  sed -i 's/use crate::hash/use crate::core::hash/g' "$f"
  sed -i 's/use crate::index/use crate::core::index/g' "$f"
  sed -i 's/use crate::store/use crate::core::store/g' "$f"
  sed -i 's/use crate::merge/use crate::core::merge/g' "$f"
  sed -i 's/use crate::reset/use crate::core::reset/g' "$f"
  sed -i 's/use crate::stash/use crate::core::stash/g' "$f"
  sed -i 's/use crate::tag/use crate::core::tag/g' "$f"
  sed -i 's/use crate::cherry_pick/use crate::core::cherry_pick/g' "$f"
  sed -i 's/use crate::bisect/use crate::core::bisect/g' "$f"
  sed -i 's/use crate::diff/use crate::core::diff/g' "$f"
  sed -i 's/use crate::status/use crate::core::status/g' "$f"
  sed -i 's/use crate::ignore/use crate::core::ignore/g' "$f"
  sed -i 's/use crate::attributes/use crate::core::attributes/g' "$f"
  sed -i 's/use crate::config/use crate::core::config/g' "$f"
  sed -i 's/use crate::auth/use crate::core::auth/g' "$f"
  sed -i 's/use crate::hooks/use crate::core::hooks/g' "$f"
done

# Fix imports in commands files
cd /home/atix/mug/src/commands
for f in *.rs; do
  sed -i 's/use crate::/use crate::core::/g' "$f"
  sed -i 's/use crate::core::commands/use crate::commands/g' "$f"
done

# Fix imports in remote files
cd /home/atix/mug/src/remote
for f in *.rs; do
  sed -i 's/use crate::error/use crate::core::error/g' "$f"
  sed -i 's/use crate::repo/use crate::core::repo/g' "$f"
  sed -i 's/use crate::commit/use crate::core::commit/g' "$f"
  sed -i 's/use crate::branch/use crate::core::branch/g' "$f"
  sed -i 's/use crate::database/use crate::core::database/g' "$f"
done

# Fix imports in main.rs
cd /home/atix/mug/src
sed -i 's/use mug::error/use mug::core::error/g' main.rs
sed -i 's/use mug::repo/use mug::core::repo/g' main.rs
sed -i 's/use mug::merge/use mug::core::merge/g' main.rs
sed -i 's/use mug::cherry_pick/use mug::core::cherry_pick/g' main.rs
sed -i 's/use mug::bisect/use mug::core::bisect/g' main.rs
sed -i 's/use mug::stash/use mug::core::stash/g' main.rs
sed -i 's/use mug::tag/use mug::core::tag/g' main.rs
sed -i 's/use mug::index/use mug::core::index/g' main.rs
sed -i 's/use mug::reset/use mug::core::reset/g' main.rs
sed -i 's/use mug::commands/use mug::commands/g' main.rs
sed -i 's/use mug::branch/use mug::core::branch/g' main.rs
sed -i 's/use mug::remote/use mug::remote/g' main.rs

echo "Imports fixed!"
