
use super::*;
use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::iter::FromIterator;

pub trait Termer<T> : Clone {
	fn termify(&self, v:&T) -> Term;
}
pub trait Determer<T> : Clone {
	fn determify(&self, v:&Term) -> Result<T, DetermifyError>;
}

#[derive(Debug)]
pub struct DetermifyError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
	pub cause:Option<Box<Error>>,
}
impl Display for DetermifyError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}
impl DetermifyError {
	pub fn new(source: &Term, msg:String) -> Self {
		let (line, column) = source.line_and_col();
		Self{ line, column, msg, cause:None }
	}
	pub fn new_with_cause(source:&Term, msg:String, cause:Option<Box<Error>>) -> Self {
		let (line, column) = source.line_and_col();
		DetermifyError{ line, column, msg, cause }
	}
}

impl Error for DetermifyError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&Error> { self.cause.as_ref().map(|e| e.as_ref()) }
}

pub trait Bitermer<T> : Termer<T> + Determer<T> {} //bidirectional termer and determer

pub trait Termable {
	fn termify(&self) -> Term;
}
pub trait Determable {
	fn determify(&Term) -> Result<Self, DetermifyError> where Self:Sized;
}


//for scanning over structs with named pairs where the order is usually the same. Guesses that each queried field will be after the last, and only does a full scanaround when it finds that's not the case. Extremely efficient, if order is maintained.
pub struct FieldScanning<'a>{
	pub v:&'a Term,
	pub li:&'a [Term],
	pub eye:usize,
}
impl<'a> FieldScanning<'a>{
	pub fn new(v:&'a Term) -> Self {
		FieldScanning{ v:v, li:v.tail().as_slice(), eye:0, }
	}
	pub fn seek(&mut self, key:&str) -> Result<&Term, DetermifyError> {
		for _ in 0..self.li.len() {
			let c = &self.li[self.eye];
			if c.initial_str() == key {
				return
					if let Some(s) = c.tail().next() {
						Ok(s)
					}else{
						Err(DetermifyError::new(c, format!("expected a subterm, but the term has no tail")))
					}
			}
			self.eye += 1;
			if self.eye >= self.li.len() { self.eye = 0; }
		}
		Err(DetermifyError::new(self.v, format!("could not find key \"{}\"", key)))
	}
}


#[derive(Clone)]
pub struct DefaultTermer();
#[derive(Clone)]
pub struct DefaultDetermer();
#[derive(Clone)]
pub struct DefaultBitermer();
impl<T> Bitermer<T> for DefaultBitermer where T:Termable + Determable {}
impl<T> Termer<T> for DefaultBitermer where T:Termable {
	fn termify(&self, v:&T) -> Term { v.termify() }
}
impl<T> Determer<T> for DefaultBitermer where T:Determable {
	fn determify(&self, v:&Term) -> Result<T, DetermifyError> { T::determify(v) }
}
impl<T> Termer<T> for DefaultTermer where T:Termable {
	fn termify(&self, v:&T) -> Term { v.termify() }
}
impl<T> Determer<T> for DefaultDetermer where T:Determable {
	fn determify(&self, v:&Term) -> Result<T, DetermifyError> { T::determify(v) }
}



pub fn termify<T>(v:&T) -> Term where T: Termable {
	v.termify()
}
pub fn determify<T>(v:&Term) -> Result<T, DetermifyError> where T: Determable {
	T::determify(v)
}

#[derive(Debug)]
pub enum TermposeError{
	ParserError(PositionedError),
	DetermifyError(DetermifyError),
}

pub fn deserialize<T>(v:&str) -> Result<T, TermposeError> where T : Determable {
	match parse(v) {
		Ok(t)=> determify(&t).map_err(|e| TermposeError::DetermifyError(e)),
		Err(e)=> Err(TermposeError::ParserError(e)),
	}
}
pub fn serialize<T>(v:&T) -> String where T: Termable {
	termify(v).to_string()
}



#[derive(Copy, Clone)]
pub struct IsizeBi();
impl Bitermer<isize> for IsizeBi {}
impl Termer<isize> for IsizeBi {
	fn termify(&self, v:&isize) -> Term { v.to_string().as_str().into() }
}
impl Determer<isize> for IsizeBi {
	fn determify(&self, v:&Term) -> Result<isize, DetermifyError> {
		isize::from_str(v.initial_str()).map_err(|er|{
			DetermifyError::new_with_cause(v, "couldn't parse isize".into(), Some(Box::new(er)))
		})
	}
}


#[derive(Copy, Clone)]
pub struct UsizeBi();
impl Bitermer<usize> for UsizeBi {}
impl Termer<usize> for UsizeBi {
	fn termify(&self, v:&usize) -> Term { v.to_string().as_str().into() }
}
impl Determer<usize> for UsizeBi {
	fn determify(&self, v:&Term) -> Result<usize, DetermifyError> {
		usize::from_str(v.initial_str()).map_err(|er|{
			DetermifyError::new_with_cause(v, "couldn't parse usize".into(), Some(Box::new(er)))
		})
	}
}


#[derive(Copy, Clone)]
pub struct BoolBi();
impl Bitermer<bool> for BoolBi {}
impl Termer<bool> for BoolBi {
	fn termify(&self, v:&bool) -> Term { v.to_string().as_str().into() }
}
impl Determer<bool> for BoolBi {
	fn determify(&self, v:&Term) -> Result<bool, DetermifyError> {
		match v.initial_str() {
			"true" | "⊤" | "yes" => {
				Ok(true)
			},
			"false" | "⟂" | "no" => {
				Ok(false)
			},
			_=> Err(DetermifyError::new_with_cause(v, "expected a bool here".into(), None))
		}
	}
}


impl Termable for String {
	fn termify(&self) -> Term {
		self.as_str().into()
	}
}
impl Determable for String {
	fn determify(v:&Term) -> Result<Self, DetermifyError> {
		match *v {
			Atomv(ref a)=> Ok(a.v.clone()),
			Listv(_)=> Err(DetermifyError::new_with_cause(v, "sought string, found list".into(), None)),
		}
	}
}



fn termify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<Term>)
	where InnerTran: Termer<T>, I:Iterator<Item=&'a T>, T:'a
{
	for vi in v { output.push(inner.termify(vi)); }
}
fn determify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<T>) -> Result<(), DetermifyError>
	where InnerTran: Determer<T>, I:Iterator<Item=&'a Term>
{
	// let errors = Vec::new();
	for vi in v {
		match inner.determify(vi) {
			Ok(vii)=> output.push(vii),
			Err(e)=> return Err(e),
		}
	}
	Ok(())
	// if errors.len() > 0 {
	// 	let msgs = String::new();
	// 	for e in errors {
	// 		msgs.push(format!("{}\n"))
	// 	}
	// }
}


#[derive(Copy, Clone)]
pub struct SequenceTran<SubTran>(SubTran);
impl<T, SubTran> Bitermer<Vec<T>> for SequenceTran<SubTran> where SubTran:Bitermer<T> {}
impl<T, SubTran> Termer<Vec<T>> for SequenceTran<SubTran> where SubTran:Termer<T> {
	fn termify(&self, v:&Vec<T>) -> Term {
		let mut ret = Vec::new();
		termify_seq_into(&self.0, v.iter(), &mut ret);
		ret.into()
	}
}
impl<T, SubTran> Determer<Vec<T>> for SequenceTran<SubTran> where SubTran:Determer<T> {
	fn determify(&self, v:&Term) -> Result<Vec<T>, DetermifyError> {
		let mut ret = Vec::new();
		try!(determify_seq_into(&self.0, v.contents(), &mut ret));
		Ok(ret)
	}
}

#[derive(Copy, Clone)]
pub struct TaggedSequenceTran<'a, SubTran>(&'a str, SubTran);
impl<'a, T, SubTran> Bitermer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Bitermer<T> {}
impl<'a, T, SubTran> Termer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Termer<T> {
	fn termify(&self, v:&Vec<T>) -> Term {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		termify_seq_into(&self.1, v.iter(), &mut ret);
		ret.into()
	}
}

fn ensure_tag<'b>(v:&'b Term, tag:&str) -> Result<std::slice::Iter<'b, Term>, DetermifyError> {
	let mut i = v.contents();
	if let Some(name_term) = i.next() {
		match *name_term {
			Atomv(ref at)=>{
				let name = at.v.as_str();
				if name == tag {
					Ok(i)
				}else{
					Err(DetermifyError::new_with_cause(name_term, format!("expected \"{}\" here, but instead there was \"{}\"", tag, name), None))
				}
			},
			_=> {
				Err(DetermifyError::new_with_cause(name_term, format!("expected \"{}\" here, but instead there was a list term", tag), None))
			}
		}
	}else{
		Err(DetermifyError::new_with_cause(v, format!("expected \"{}\" at beginning, but the term was empty", tag), None))
	}
}

impl<'a, T, SubTran> Determer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Determer<T> {
	fn determify(&self, v:&Term) -> Result<Vec<T>, DetermifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, &self.0)?;
		determify_seq_into(&self.1, it, &mut ret)?;
		Ok(ret)
	}
}


fn determify_pair<K, V, KeyTran, ValTran>(kt:&KeyTran, vt:&ValTran, v:&Term) -> Result<(K,V), DetermifyError>
	where KeyTran:Determer<K>, ValTran:Determer<V>
{
	match *v {
		Listv(ref lc)=> {
			if lc.v.len() == 2 {
				unsafe{
					let k = kt.determify(lc.v.get_unchecked(0))?; // safe: we just checked the length
					let v = vt.determify(lc.v.get_unchecked(1))?; //
					Ok((k, v))
				}
			}else{
				Err(DetermifyError::new_with_cause(v, format!("expected a pair, two elements, but the list here has {}", lc.v.len()), None))
			}
		}
		Atomv(_)=> {
			Err(DetermifyError::new_with_cause(v, "expected a pair, but the term here is an atom".into(), None))
		}
	}
}

#[derive(Copy, Clone)]
pub struct PairBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Bitermer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Bitermer<K>, ValTran:Bitermer<V> {}
impl<K, V, KeyTran, ValTran> Termer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Termer<K>, ValTran:Termer<V> {
	fn termify(&self, v:&(K,V)) -> Term {
		let kt = self.0.termify(&v.0);
		let vt = self.1.termify(&v.1);
		list!(kt, vt).into()
	}
}
impl<K, V, KeyTran, ValTran> Determer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Determer<K>, ValTran:Determer<V> {
	fn determify(&self, v:&Term) -> Result<(K,V), DetermifyError> {
		determify_pair(&self.0, &self.1, v)
	}
}

fn termify_map<'a, K, V, KeyTermer, ValTermer, I>(ktr:&KeyTermer, vtr:&ValTermer, i:I, o:&mut Vec<Term>)
	where
		KeyTermer: Termer<K>,
		ValTermer: Termer<V>,
		I: Iterator<Item=(&'a K, &'a V)>,
		K: 'a,
		V: 'a,
{
	for (kr, vr) in i {
		o.push(list!(ktr.termify(kr), vtr.termify(vr)).into())
	}
}

fn determify_map<'a, K, V, KeyTran, ValTran, I>(ktr:&KeyTran, vtr:&ValTran, i:I, o:&mut Vec<(K, V)>) -> Result<(), DetermifyError>
	where
		KeyTran: Determer<K>,
		ValTran: Determer<V>,
		I: Iterator<Item=&'a Term>,
{
	for v in i {
		o.push(determify_pair(ktr, vtr, v)?);
	}
	Ok(())
}

impl<K, V> Termable for HashMap<K, V> where
	K: Eq + Hash + Termable,
	V: Eq + Hash + Termable,
{
	fn termify(&self) -> Term {
		let mut ret = Vec::new();
		termify_map(&DefaultTermer(), &DefaultTermer(), self.iter(), &mut ret);
		ret.into()
	}
}
impl<K, V> Determable for HashMap<K, V>
	where
		K: Eq + Hash + Determable,
		V: Eq + Hash + Determable,
{
	fn determify(v:&Term) -> Result<HashMap<K,V>, DetermifyError> {
		let mut ret = Vec::new();
		determify_map(&DefaultDetermer(), &DefaultDetermer(), v.contents(), &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}

#[derive(Clone)]
pub struct HashMapBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Bitermer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Bitermer<K>, ValTran:Bitermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{}
impl<K, V, KeyTran, ValTran> Termer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Termer<K>, ValTran:Termer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn termify(&self, v:&HashMap<K, V>) -> Term {
		let mut ret = Vec::new();
		termify_map(&self.0, &self.1, v.iter(), &mut ret);
		ret.into()
	}
}
impl<K, V, KeyTran, ValTran> Determer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Determer<K>, ValTran:Determer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn determify(&self, v:&Term) -> Result<HashMap<K,V>, DetermifyError> {
		let mut ret = Vec::new();
		determify_map(&self.0, &self.1, v.contents(), &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}

#[derive(Clone)]
pub struct TaggedHashMapBi<'a, KeyTran, ValTran>(&'a str, KeyTran, ValTran);
impl<'a, K, V, KeyTran, ValTran> Bitermer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Bitermer<K>, ValTran:Bitermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{}
impl<'a, K, V, KeyTran, ValTran> Termer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Termer<K>, ValTran:Termer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn termify(&self, v:&HashMap<K, V>) -> Term {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		termify_map(&self.1, &self.2, v.iter(), &mut ret);
		ret.into()
	}
}
impl<'a, K, V, KeyTran, ValTran> Determer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Determer<K>, ValTran:Determer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn determify(&self, v:&Term) -> Result<HashMap<K,V>, DetermifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, self.0)?;
		determify_map(&self.1, &self.2, it, &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn idempotent_int() {
		assert!(90isize == IsizeBi().determify(&IsizeBi().termify(&90isize)).unwrap());
	}
	
	#[test]
	fn tricky_list_parse() {
		let listo = list!(list!("tricky", "list"), list!("parse"));
		let tranner:SequenceTran<SequenceTran<DefaultBitermer>> = SequenceTran(SequenceTran(DefaultBitermer()));
		let lv:Vec<Vec<String>> = tranner.determify(&listo).unwrap();
		assert!(lv.len() == 2);
		assert!(lv[0].len() == 2);
		assert!(lv[0][0].len() == 6);
		assert!(tranner.termify(&lv).to_string() == "((tricky list) (parse))");
	}
	
	#[test]
	fn do_hash_map() {
		let t = parse("a:b c:d d:e e:f").unwrap();
		let bt = TaggedHashMapBi("ob", DefaultBitermer(), DefaultBitermer());
		let utr: Result<HashMap<String, String>, DetermifyError> = bt.determify(&t);
		assert!(utr.is_err());
		let tt = parse("ob a:b c:d d:e e:f").unwrap();
		let hm:HashMap<String, String> = TaggedHashMapBi("ob", DefaultBitermer(), DefaultBitermer()).determify(&tt).unwrap();
		assert!(hm.get("a").unwrap() == "b");
		assert!(hm.get("d").unwrap() == "e");
	}
	
	fn give_hm() -> HashMap<String, String> {
		[
			("a".into(), "b".into()),
			("c".into(), "d".into()),
		].iter().cloned().collect()
	}
	
	#[test]
	fn implicit_bitermers() {
		let t = parse("a:b c:d").unwrap();
		let exh = give_hm();
		
		let exu:HashMap<String, String> = determify(&t).unwrap();
		
		assert!(exu == exh)
	}
	
	#[test]
	fn automatic_deserialize() {
		let hm:HashMap<String, String> = deserialize("a:b c:d").unwrap();
		assert!(hm == give_hm());
	}
	
	#[test]
	fn automatic_serialize() {
		let hm = give_hm();
		let cln = deserialize(&serialize(&hm)).unwrap();
		assert!(hm == cln);
	}
}