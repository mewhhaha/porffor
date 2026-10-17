use super::*;
#[derive(Debug)]
pub(crate) struct Site {
    pub image: u32,
    pub factory: u32,
}
impl Plan {
    pub(super) fn collect_sites(
        &mut self,
        operators: &[(usize, Operator<'_>)],
    ) -> Result<(), NativeProgramTemplateError> {
        for (index, (offset, operator)) in operators.iter().enumerate() {
            match operator {
                Operator::ArrayNewData {
                    array_type_index: target,
                    array_data_index: segment,
                } => {
                    let width = *self
                        .arrays
                        .get(target)
                        .ok_or_else(|| invalid("nonpacked data target"))?;
                    let image = self.segment(*segment)?;
                    let whole = if index >= 2 {
                        match (&operators[index - 2].1, &operators[index - 1].1) {
                            (
                                Operator::I32Const { value: 0 },
                                Operator::I32Const { value: length },
                            ) if *length >= 0 => {
                                (*length as usize).checked_mul(width.bytes())
                                    == Some(self.images[image as usize].bytes.len())
                            }
                            _ => false,
                        }
                    } else {
                        false
                    };
                    let construction = if whole {
                        NativeProgramArrayConstruction::Whole
                    } else {
                        NativeProgramArrayConstruction::Slice
                    };
                    if whole {
                        self.suppressed.insert(operators[index - 2].0);
                        self.suppressed.insert(operators[index - 1].0);
                    }
                    let factory = self.factory(*target, width, construction);
                    self.sites.insert(*offset, Site { image, factory });
                }
                Operator::ArrayNewFixed {
                    array_type_index: target,
                    array_size: count,
                } => {
                    let Some(width) = self.arrays.get(target).copied() else {
                        continue;
                    };
                    let count = *count as usize;
                    if count == 0 {
                        // No source operands to relocate; retain the original
                        // fresh zero-length allocation at its actual Wasm site.
                        continue;
                    }
                    if count > index {
                        continue;
                    }
                    let mut bytes = Vec::new();
                    let mut valid = true;
                    for (position, op) in &operators[index - count..index] {
                        if self.suppressed.contains(position) {
                            valid = false;
                            break;
                        }
                        if let Operator::I32Const { value } = op {
                            match width {
                                NativeProgramArrayWidth::I8 => bytes.push(*value as u8),
                                NativeProgramArrayWidth::I16 => {
                                    bytes.extend_from_slice(&(*value as u16).to_le_bytes())
                                }
                            }
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    for (position, _) in &operators[index - count..index] {
                        self.suppressed.insert(*position);
                    }
                    let image = self.image(bytes);
                    let factory =
                        self.factory(*target, width, NativeProgramArrayConstruction::Whole);
                    self.sites.insert(*offset, Site { image, factory });
                }
                _ => {}
            }
        }
        Ok(())
    }
}
