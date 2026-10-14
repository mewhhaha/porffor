//! Checked, private passive recipes for large pure fixed Array literals.
use super::*;
pub(crate) const COMPACT_ARRAY_RECIPE_SEGMENT: u32 = 3;
pub(crate) const COMPACT_ARRAY_RECIPE_ROW_BYTES: u32 = 16;
const MIN_COMPACT_NODES: usize = 128;

macro_rules! compact_array_opcode_domain {
    ($($kind:ident = $word:literal),+ $(,)?) => {
        #[derive(Clone, Copy)]
        pub(crate) enum CompactArrayOpcode { $($kind),+ }
        impl CompactArrayOpcode {
            pub(crate) const ALL: &'static [Self] = &[$(Self::$kind),+];
            pub(crate) const fn word(self) -> u32 { match self { $(Self::$kind => $word),+ } }
        }
    };
}
compact_array_opcode_domain!(
    Begin = 0,
    End = 1,
    Hole = 2,
    Undefined = 3,
    Null = 4,
    Boolean = 5,
    Number = 6,
    String = 7
);

pub(crate) enum CompactArrayNode<'a> {
    Begin(u32),
    End,
    Hole,
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    String(&'a str),
}
impl CompactArrayNode<'_> {
    fn encoded(
        &self,
        string_index: impl FnOnce(&str) -> Result<PooledStringIndex, EmitError>,
    ) -> Result<(CompactArrayOpcode, u64), EmitError> {
        Ok(match self {
            Self::Begin(length) => (CompactArrayOpcode::Begin, u64::from(*length)),
            Self::End => (CompactArrayOpcode::End, 0),
            Self::Hole => (CompactArrayOpcode::Hole, 0),
            Self::Undefined => (CompactArrayOpcode::Undefined, 0),
            Self::Null => (CompactArrayOpcode::Null, 0),
            Self::Boolean(value) => (CompactArrayOpcode::Boolean, u64::from(*value)),
            Self::Number(bits) => (CompactArrayOpcode::Number, *bits),
            Self::String(value) => (
                CompactArrayOpcode::String,
                u64::from(string_index(value)?.ordinal()),
            ),
        })
    }
}
/// Only this complete pure walk can mint a recipe; effectful IR has no recipe.
pub(crate) struct CompactArrayPlan<'a> {
    nodes: Vec<CompactArrayNode<'a>>,
    depth: u32,
}
impl<'a> CompactArrayPlan<'a> {
    pub(crate) fn of(elements: &'a [TypedExpr]) -> Option<Self> {
        let mut nodes = vec![CompactArrayNode::Begin(u32::try_from(elements.len()).ok()?)];
        let mut frames = vec![(elements, 0usize)];
        let mut depth = 1u32;
        while let Some((values, next)) = frames.last().copied() {
            if next == values.len() {
                nodes.push(CompactArrayNode::End);
                frames.pop();
                continue;
            }
            frames.last_mut().expect("current frame").1 += 1;
            let value = &values[next];
            let node = match &value.expr {
                ExprIr::ArrayLiteral(children) => {
                    let length = u32::try_from(children.len()).ok()?;
                    frames.push((children, 0));
                    depth = depth.max(u32::try_from(frames.len()).ok()?);
                    CompactArrayNode::Begin(length)
                }
                ExprIr::ArrayHole => CompactArrayNode::Hole,
                ExprIr::Undefined => CompactArrayNode::Undefined,
                ExprIr::Null => CompactArrayNode::Null,
                ExprIr::Boolean(value) => CompactArrayNode::Boolean(*value),
                ExprIr::Number(bits) => CompactArrayNode::Number(*bits),
                ExprIr::String(value) => CompactArrayNode::String(value),
                // BigInt/Symbol, reads, calls, spread and suspension retain the ordinary emitter.
                _ => return None,
            };
            nodes.push(node);
        }
        if nodes.len() < MIN_COMPACT_NODES {
            return None;
        }
        u32::try_from(
            nodes
                .len()
                .checked_mul(COMPACT_ARRAY_RECIPE_ROW_BYTES as usize)?,
        )
        .ok()?;
        Some(Self { nodes, depth })
    }
    pub(crate) fn nodes(&self) -> &[CompactArrayNode<'a>] {
        &self.nodes
    }
    fn encode(
        &self,
        mut string_index: impl FnMut(&str) -> Result<PooledStringIndex, EmitError>,
    ) -> Result<Vec<u8>, EmitError> {
        let mut bytes =
            Vec::with_capacity(self.nodes.len() * COMPACT_ARRAY_RECIPE_ROW_BYTES as usize);
        for node in &self.nodes {
            let (kind, payload) = node.encoded(&mut string_index)?;
            bytes.extend_from_slice(&kind.word().to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&payload.to_le_bytes());
        }
        Ok(bytes)
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct CompactArrayRecipe {
    offset: u32,
    bytes: u32,
    depth: u32,
}
impl CompactArrayRecipe {
    pub(crate) const fn offset(self) -> u32 {
        self.offset
    }
    pub(crate) const fn bytes(self) -> u32 {
        self.bytes
    }
    pub(crate) const fn depth(self) -> u32 {
        self.depth
    }
}
#[derive(Default, Debug)]
pub(super) struct CompactArrayRecipes {
    bytes: Vec<u8>,
    rows: BTreeMap<Vec<u8>, CompactArrayRecipe>,
    error: Option<EmitError>,
}
impl CompactArrayRecipes {
    pub(super) fn collect(
        &mut self,
        plan: &CompactArrayPlan<'_>,
        string_index: impl FnMut(&str) -> Result<PooledStringIndex, EmitError>,
    ) {
        let result = (|| {
            let key = plan.encode(string_index)?;
            if self.rows.contains_key(&key) {
                return Ok(());
            }
            let offset = u32::try_from(self.bytes.len()).ok();
            let bytes = u32::try_from(key.len()).ok();
            let Some((offset, bytes)) = offset
                .zip(bytes)
                .filter(|(offset, bytes)| offset.checked_add(*bytes).is_some())
            else {
                return Err(EmitError::unsupported(
                    "compact Array literal recipes exceed the passive data address domain",
                ));
            };
            self.bytes.extend_from_slice(&key);
            self.rows.insert(
                key,
                CompactArrayRecipe {
                    offset,
                    bytes,
                    depth: plan.depth,
                },
            );
            Ok(())
        })();
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
    }
    pub(super) fn find(
        &self,
        plan: &CompactArrayPlan<'_>,
        string_index: impl FnMut(&str) -> Result<PooledStringIndex, EmitError>,
    ) -> Result<CompactArrayRecipe, EmitError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        self.rows
            .get(&plan.encode(string_index)?)
            .copied()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "compiler invariant: an admitted compact Array recipe was not collected",
                )
            })
    }
    pub(super) fn has_recipes(&self) -> bool {
        !self.rows.is_empty() || self.error.is_some()
    }
    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
impl StringPool {
    pub(crate) fn compact_array_recipe(
        &self,
        elements: &[TypedExpr],
    ) -> Result<Option<CompactArrayRecipe>, EmitError> {
        CompactArrayPlan::of(elements)
            .map(|plan| {
                self.compact_array_recipes
                    .find(&plan, |text| self.pooled_string_index(text))
            })
            .transpose()
    }
    pub(crate) fn has_compact_array_recipes(&self) -> bool {
        self.compact_array_recipes.has_recipes()
    }
}

#[cfg(test)]
mod tests;
