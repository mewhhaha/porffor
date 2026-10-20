for (var name of [
  'Basic_Emoji', 'Emoji_Keycap_Sequence', 'RGI_Emoji_Modifier_Sequence',
  'RGI_Emoji_Flag_Sequence', 'RGI_Emoji_Tag_Sequence',
  'RGI_Emoji_ZWJ_Sequence', 'RGI_Emoji'
]) {
  rejects(propertyUnits(name, 112), [117]);
  rejects([91].concat(propertyUnits(name, 112), [93]), [117]);
  rejects(propertyUnits(name, 80), [118]);
  rejects([91].concat(propertyUnits(name, 80), [93]), [118]);
  rejects([91, 94].concat(propertyUnits(name, 112), [93]), [118]);
}
rejects([91, 94].concat(propertyUnits('ASCII', 112), propertyUnits('Basic_Emoji', 112), [93]), [118]);
rejects(propertyUnits('Basic_Emoji', 112).concat(propertyUnits('No_Such_Property', 112)), [118]);
rejects(propertyUnits('No_Such_Property', 112).concat(propertyUnits('Basic_Emoji', 112)), [118]);
rejects(propertyUnits('Basic_emoji', 112), [118]);
rejects(propertyUnits('RGI_Emoji_Unknown', 112), [118]);
var positiveString = propertyUnits('Basic_Emoji', 112);
rejects(positiveString.concat([41]), [118]);
rejects(positiveString.concat([40]), [118]);
rejects(positiveString.concat([42, 42]), [118]);
rejects(positiveString.concat([92, 107, 60, 109, 105, 115, 115, 105, 110, 103, 62]), [118]);
print('regexp-property-strings:ok');
262;
