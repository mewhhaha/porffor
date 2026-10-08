#!/usr/bin/env python3
"""Genuine CLDR47 DurationFormat source foundation; native execution pending."""
import argparse,gzip,hashlib,json,re,tarfile
from pathlib import Path
import xml.etree.ElementTree as ET
from intl_cldr_profile import CLDR_COMMIT,CldrProfile,INHERIT,NO_INHERIT,ResolvedLeaf,Segment,normalize_path,parse_path,path_text
ROOT=Path(__file__).resolve().parents[1]
SOURCE='crates/lila-intl/data/duration-cldr-47'
PROFILE='crates/lila-intl/src/duration_format/generated/profile.json'
LOCALES=tuple(sorted(('en','en-US','ar','ar-EG','zh','zh-Hans','zh-Hans-CN','de','fr','it','ja','ko','hi','es','sr')))
UNITS=('year','month','week','day','hour','minute','second','millisecond','microsecond','nanosecond')
STYLES=('long','short','narrow');COUNTS=('zero','one','two','few','many','other')
ARCHIVE_SHA='f85496f1e76ca0a12a0d7603c5f3243a94e2506926176f6ec7fe178327ffcebf'
MANIFEST_SHA='582f922168284a76255554a4b74c6200f0e954853ef2eaf49bceb393a2d48b49'
def encoded(value):return (json.dumps(value,indent=2,ensure_ascii=False,sort_keys=True)+'\n').encode()
def sha(raw):return hashlib.sha256(raw).hexdigest()
def capture(root):
    base=root/'crates/lila-intl/data/number-cldr-47';raw=(base/'source-manifest.json').read_bytes()
    if sha(raw)!=MANIFEST_SHA or sha((base/'sources.tar.gz').read_bytes())!=ARCHIVE_SHA:raise ValueError('CLDR47 authority changed')
    authority={r['path']:r for r in json.loads(raw)['files']}
    with tarfile.open(base/'sources.tar.gz') as archive:
        def read(name):
            raw=archive.extractfile(name).read();row=authority[name]
            if sha(raw)!=row['sha256'] or len(raw)!=row['bytes']:raise ValueError('archive member changed: '+name)
            return raw
        parents={}
        for group in ET.fromstring(read('reference/cldr/common/supplemental/supplementalData.xml')).findall('./parentLocales'):
            if group.get('component') is None:
                for entry in group.findall('parentLocale'):
                    for locale in entry.attrib['locales'].split():parents[locale]=entry.attrib['parent']
        physical={'root'}
        for locale in LOCALES:
            locale=locale.replace('-','_')
            while locale!='root':
                if locale in physical:break
                physical.add(locale);locale=parents.get(locale,locale.rsplit('_',1)[0] if '_' in locale else 'root')
        names=['LICENSE','common/dtd/ldml.dtd','common/supplemental/supplementalData.xml','common/supplemental/supplementalMetadata.xml',
               'docs/ldml/tr35.md','docs/ldml/tr35-general.md','docs/ldml/tr35-numbers.md']+['common/main/'+l+'.xml' for l in sorted(physical)]
        files={name:read('reference/cldr/'+name) for name in names}
    files['selector.json']=encoded({'schema_version':1,'release':'47.0.0','commit':CLDR_COMMIT,
        'minimum_draft':'contributed','default_locale':'en-US','locales':list(LOCALES)})
    rows=[{'path':n,'bytes':len(r),'sha256':sha(r),'kind':'authored-selector' if n=='selector.json' else 'primary-CLDR47',
        **({} if n=='selector.json' else {'archive_path':'reference/cldr/'+n})} for n,r in sorted(files.items())]
    files['manifest.json']=encoded({'schema':1,'release':'47.0.0','commit':CLDR_COMMIT,'files':rows,
        'total_bytes':sum(r['bytes'] for r in rows),'archive_sha256':ARCHIVE_SHA,'source_manifest_sha256':MANIFEST_SHA})
    return files

def resolve_count(profile,locale,path,seen=()):
    """Pinned LDML count fallback locally precedes parent inheritance."""
    path=normalize_path(parse_path(path) if isinstance(path,str) else path,profile.schema)
    if path[-1].tag!='unitPattern' or path[-1].get('count') not in COUNTS:raise ValueError('closed unit count required')
    key=locale,path
    if key in seen:raise ValueError('cyclic unit alias')
    other=(*path[:-1],Segment('unitPattern',tuple(sorted((k,'other' if k=='count' else v) for k,v in path[-1].attributes))))
    for ancestor in profile.lineage(locale):
        tree=profile.locales[ancestor]
        for candidate in ((path,) if path==other else (path,other)):
            leaf=tree.leaves.get(candidate)
            if leaf and leaf.text==NO_INHERIT:return None
            if leaf and leaf.text!=INHERIT:
                if not leaf.text:raise ValueError('empty unit leaf')
                return ResolvedLeaf(leaf.text,leaf.attributes,ancestor,candidate)
        redirected=tree.redirect(path)
        if redirected is not None:return resolve_count(profile,locale,redirected,(*seen,key))
    return None

def text(profile,consumed,locale,path,count=False):
    path=parse_path(path);leaf=resolve_count(profile,locale,path) if count else profile.resolve(locale,path)
    if leaf is None or leaf.attributes or leaf.value in ('',INHERIT,NO_INHERIT):raise ValueError('unresolved/qualified leaf: '+path_text(path))
    consumed[locale+'/'+path_text(path)]={'source_locale':leaf.source_locale,'source_path':path_text(leaf.source_path),
        'value':leaf.value,'value_attributes':dict(leaf.attributes)}
    return leaf.value

def placeholders(value,slots,optional=False):
    rest=value
    for slot in slots:
        token='{'+str(slot)+'}';count=value.count(token)
        if count!=1 and not(optional and count==0):raise ValueError('placeholder multiplicity')
        rest=rest.replace(token,'')
    if '{' in rest or '}' in rest:raise ValueError('unconsumed placeholder')
    return value

def digital(value,skeleton):
    runs=list(re.finditer(r'[hHms]+',value));widths=[];separators=[]
    if len(runs)!=len(skeleton) or value[:runs[0].start()] or value[runs[-1].end():]:raise ValueError('digital field/endpoint domain')
    for i,(run,field) in enumerate(zip(runs,skeleton)):
        if set(run.group().lower())!={field} or len(run.group()) not in (1,2):raise ValueError('digital width')
        widths.append(len(run.group()))
        if i:
            sep=value[runs[i-1].end():run.start()]
            if not sep or "'" in sep or any(c.isascii() and c.isalpha() for c in sep):raise ValueError('digital separator')
            separators.append(sep)
    return {'widths':widths,'separators':separators}

def number_patterns(nf,locale,unit,style):
    t=nf['tables'];row=t['profiles'][nf['locales'][locale]['profile']]
    choice=t['unit_sets'][row['units'][{'short':0,'narrow':1,'long':2}[style]]]['simple'][nf['sanctioned_units'].index(unit)]['choices']
    result=[]
    for index in t['choices'][choice]['values'][:6]:
        parts=[]
        for kind,value in t['patterns'][index]:
            if kind=='Number':parts.append('{0}')
            elif kind in ('Unit','Literal'):parts.append(t['strings'][value])
            else:raise ValueError('unexpected shared NF unit token')
        result.append(''.join(parts))
    return result

def generate(root,profile):
    if profile.selector['locales']!=list(LOCALES) or profile.selector['default_locale']!='en-US':raise ValueError('exact15 locale recipe required')
    nf=json.loads(gzip.decompress((root/'crates/lila-intl/data/number-cldr-47/profiles.json.gz').read_bytes()));consumed={};locales=[]
    for name in LOCALES:
        locale=name.replace('-','_');units=[];lists=[];patterns={}
        for unit in UNITS:
            for style in STYLES:
                values=[placeholders(text(profile,consumed,locale,f"units/unitLength[@type='{style}']/unit[@type='duration-{unit}']/unitPattern[@count='{c}']",True),[0],True) for c in COUNTS]
                if values!=number_patterns(nf,name,unit,style):raise ValueError('primary Duration/NF unit mismatch: '+name+'/'+unit+'/'+style)
                units.append({'unit':unit,'style':style,'patterns':values})
        for style in STYLES:
            kind='unit' if style=='long' else 'unit-'+style
            templates=[placeholders(text(profile,consumed,locale,f"listPatterns/listPattern[@type='{kind}']/listPatternPart[@type='{p}']"),[0,1]) for p in ('2','start','middle','end')]
            lists.append({'style':style,'patterns':templates})
        for skeleton in ('hm','hms','ms'):
            value=text(profile,consumed,locale,f"units/durationUnit[@type='{skeleton}']/durationUnitPattern")
            patterns[skeleton]={'pattern':value,**digital(value,skeleton)}
        if patterns['hm']['separators'][0]!=patterns['hms']['separators'][0] or patterns['ms']['separators'][0]!=patterns['hms']['separators'][1]:raise ValueError('digital separators disagree')
        locales.append({'locale':name,'units':units,'lists':lists,'digital':patterns})
    output={'schema':1,'cldr_commit':CLDR_COMMIT,'default_locale':'en-US','locales':locales}
    report={'schema':1,'source_only':True,'native_admission':False,'locale_count':15,'unit_style_rows':450,
        'numeric_plural_patterns_compared_to_NF':2700,'digital_patterns':45,'list_templates':180,
        'text_normalization':'none','digital_policy':'literal durationUnit separators; no timeSeparator substitution defined in LDML47',
        'profile_sha256':sha(encoded(output)),'consumed_leaves':dict(sorted(consumed.items()))}
    return output,report

def inputs(root,profile):
    names={'scripts/generate-intl-duration-profile.py','scripts/intl_cldr_profile.py','scripts/intl_ldml_schema.py','scripts/intl_positional_numbering.py',
        'scripts/intl_calendar_eras.py',SOURCE+'/spec-reference.json',SOURCE+'/manifest.json','crates/lila-intl/data/number-cldr-47/source-manifest.json',
        'crates/lila-intl/data/number-cldr-47/sources.tar.gz','crates/lila-intl/data/number-cldr-47/profiles.json.gz',
        'crates/lila-intl/data/number-cldr-47/profiles.bin',
        'crates/lila-intl/data/list-icu-2/source-inputs.json','crates/lila-intl/data/list-icu-2/sources.tar.gz'}
    names.update(SOURCE+'/'+n for n in profile.sources);supp='crates/lila-intl/data/numbering-tols-cldr-48'
    manifest=json.loads((root/supp/'source-manifest.json').read_bytes());names.add(supp+'/source-manifest.json');names.update(supp+'/'+r['path'] for r in manifest['files'])
    rows=[]
    for name in sorted(names):
        raw=(root/name).read_bytes();rows.append({'path':name,'bytes':len(raw),'sha256':sha(raw)})
    return {'schema':1,'cldr_commit':CLDR_COMMIT,'files':rows,'total_bytes':sum(r['bytes'] for r in rows)}

def put(path,raw,check):
    if check:
        if not path.exists() or path.read_bytes()!=raw:raise ValueError('generated artifact changed: '+str(path))
    else:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
def main():
    p=argparse.ArgumentParser();p.add_argument('--capture',action='store_true');p.add_argument('--check',action='store_true');p.add_argument('--repository',type=Path,default=ROOT)
    a=p.parse_args();root=a.repository.resolve()
    if a.capture:
        for name,raw in capture(root).items():put(root/SOURCE/name,raw,a.check)
    profile=CldrProfile(root/SOURCE);output,report=generate(root,profile)
    for name,value in [(PROFILE,output),(SOURCE+'/report.json',report),(SOURCE+'/source-inputs.json',inputs(root,profile))]:put(root/name,encoded(value),a.check)
    print(json.dumps({'source_only':True,'locales':15,'unit_patterns':2700,'primary_leaves':len(report['consumed_leaves']),'check':a.check}))
if __name__=='__main__':main()
