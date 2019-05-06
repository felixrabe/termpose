
use super::*;
use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::iter::FromIterator;

pub trait Wooder<T> {
	fn woodify(&self, v:&T) -> Wood;
}
pub trait Dewooder<T> {
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError>;
}

#[derive(Debug)]
pub struct DewoodifyError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
	pub cause:Option<Box<Error>>,
}
impl Display for DewoodifyError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}
impl DewoodifyError {
	pub fn new(source: &Wood, msg:String) -> Self {
		let (line, column) = source.line_and_col();
		Self{ line, column, msg, cause:None }
	}
	pub fn new_with_cause(source:&Wood, msg:String, cause:Option<Box<Error>>) -> Self {
		let (line, column) = source.line_and_col();
		DewoodifyError{ line, column, msg, cause }
	}
}

impl Error for DewoodifyError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&Error> { self.cause.as_ref().map(|e| e.as_ref()) }
}

pub trait Biwooder<T> : Wooder<T> + Dewooder<T> {} //bidirectional wooder and dewooder

impl<T, X> Biwooder<T> for X where X:Wooder<T> + Dewooder<T> {}

pub trait Woodable {
	fn woodify(&self) -> Wood;
}
pub trait Dewoodable {
	fn dewoodify(&Wood) -> Result<Self, DewoodifyError> where Self:Sized;
}

macro_rules! do_basic_stringifying_woodable_for {
	($Type:ident) => (
		impl Woodable for $Type {
			fn woodify(&self) -> Wood { self.to_string().into() }
		}
	)
}
macro_rules! do_basic_destringifying_dewoodable_for {
	($Type:ident) => (
		impl Dewoodable for $Type {
			fn dewoodify(v:&Wood) -> Result<$Type, DewoodifyError> {
				$Type::from_str(v.initial_str()).map_err(|er|{
					DewoodifyError::new_with_cause(v, format!("couldn't parse {}", stringify!($name)), Some(Box::new(er)))
				})
			}
		}
	)
}

//for scanning over structs with named pairs where the order is usually the same. Guesses that each queried field will be after the last, and only does a full scanaround when it finds that's not the case. Extremely efficient, if order is maintained.
pub struct FieldScanning<'a>{
	pub v:&'a Wood,
	pub li:&'a [Wood],
	pub eye:usize,
}
impl<'a> FieldScanning<'a>{
	pub fn new(v:&'a Wood) -> Self {
		FieldScanning{ v:v, li:v.tail().as_slice(), eye:0, }
	}
	pub fn seek(&mut self, key:&str) -> Result<&Wood, DewoodifyError> {
		for _ in 0..self.li.len() {
			let c = &self.li[self.eye];
			if c.initial_str() == key {
				return
					if let Some(s) = c.tail().next() {
						Ok(s)
					}else{
						Err(DewoodifyError::new(c, format!("expected a subwood, but the wood has no tail")))
					}
			}
			self.eye += 1;
			if self.eye >= self.li.len() { self.eye = 0; }
		}
		Err(DewoodifyError::new(self.v, format!("could not find key \"{}\"", key)))
	}
}


#[derive(Clone)]
pub struct DefaultWooder;
#[derive(Clone)]
pub struct DefaultDewooder;
#[derive(Clone)]
pub struct DefaultBiwooder;
impl<T> Wooder<T> for DefaultBiwooder where T:Woodable {
	fn woodify(&self, v:&T) -> Wood { v.woodify() }
}
impl<T> Dewooder<T> for DefaultBiwooder where T:Dewoodable {
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError> { T::dewoodify(v) }
}
impl<T> Wooder<T> for DefaultWooder where T:Woodable {
	fn woodify(&self, v:&T) -> Wood { v.woodify() }
}
impl<T> Dewooder<T> for DefaultDewooder where T:Dewoodable {
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError> { T::dewoodify(v) }
}



pub fn woodify<T>(v:&T) -> Wood where T: Woodable {
	v.woodify()
}
pub fn dewoodify<T>(v:&Wood) -> Result<T, DewoodifyError> where T: Dewoodable {
	T::dewoodify(v)
}

#[derive(Debug)]
pub enum WoodposeError{
	ParserError(PositionedError),
	DewoodifyError(DewoodifyError),
}

pub fn deserialize<T>(v:&str) -> Result<T, WoodposeError> where T : Dewoodable {
	match parse_termpose(v) {
		Ok(t)=> dewoodify(&t).map_err(|e| WoodposeError::DewoodifyError(e)),
		Err(e)=> Err(WoodposeError::ParserError(e)),
	}
}
pub fn serialize<T>(v:&T) -> String where T: Woodable {
	woodify(v).to_string()
}

do_basic_stringifying_woodable_for!(char);
do_basic_destringifying_dewoodable_for!(char);
do_basic_stringifying_woodable_for!(u32);
do_basic_destringifying_dewoodable_for!(u32);
do_basic_stringifying_woodable_for!(u64);
do_basic_destringifying_dewoodable_for!(u64);
do_basic_stringifying_woodable_for!(i32);
do_basic_destringifying_dewoodable_for!(i32);
do_basic_stringifying_woodable_for!(i64);
do_basic_destringifying_dewoodable_for!(i64);
do_basic_stringifying_woodable_for!(f32);
do_basic_destringifying_dewoodable_for!(f32);
do_basic_stringifying_woodable_for!(f64);
do_basic_destringifying_dewoodable_for!(f64);
do_basic_stringifying_woodable_for!(isize);
do_basic_destringifying_dewoodable_for!(isize);
do_basic_stringifying_woodable_for!(usize);
do_basic_destringifying_dewoodable_for!(usize);

do_basic_stringifying_woodable_for!(bool);
impl Dewoodable for bool {
	fn dewoodify(v:&Wood) -> Result<Self, DewoodifyError> {
		match v.initial_str() {
			"true" | "⊤" | "yes" => {
				Ok(true)
			},
			"false" | "⟂" | "no" => {
				Ok(false)
			},
			_=> Err(DewoodifyError::new_with_cause(v, "expected a bool here".into(), None))
		}
	}
}

// impl<I, T> Woodable for I where I: Iterator<Item=T>, T:Woodable {
	
// }

impl Woodable for String {
	fn woodify(&self) -> Wood {
		self.as_str().into()
	}
}
impl Dewoodable for String {
	fn dewoodify(v:&Wood) -> Result<Self, DewoodifyError> {
		match *v {
			Leafv(ref a)=> Ok(a.v.clone()),
			Branchv(_)=> Err(DewoodifyError::new_with_cause(v, "sought string, found branch".into(), None)),
		}
	}
}



pub struct LambdaWooder<L>(L);
impl<W, L> Wooder<W> for LambdaWooder<L> where L:Fn(&W)-> Wood {
	fn woodify(&self, v:&W) -> Wood {
		self.0(v)
	}
}
pub struct LambdaDewooder<D>(D);
impl<D, L> Dewooder<D> for LambdaDewooder<L> where L:Fn(&Wood)-> Result<D, DewoodifyError> {
	fn dewoodify(&self, v:&Wood) -> Result<D, DewoodifyError> {
		self.0(v)
	}
}

pub struct CompositeBiwooder<W, D>(W, D);
impl<T, W, D> Wooder<T> for CompositeBiwooder<W, D>
	where W:Wooder<T>
{
	fn woodify(&self, v:&T) -> Wood { self.0.woodify(v) }
}
impl<T, W, D> Dewooder<T> for CompositeBiwooder<W, D>
	where D:Dewooder<T>
{
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError> {
		Dewooder::dewoodify(&self.1, v)
	}
}


/// you might want this for initializing biwooders that can't be constexprs
pub struct OptionalBoxBiwooder<B:?Sized>(Option<Box<B>>);
impl<T, B> Wooder<T> for OptionalBoxBiwooder<B>
	where B:Wooder<T> + ?Sized
{
	fn woodify(&self, v:&T) -> Wood { self.0.as_ref().unwrap().woodify(v) }
}
impl<T, B> Dewooder<T> for OptionalBoxBiwooder<B>
	where B:Dewooder<T> + ?Sized
{
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError> {
		Dewooder::dewoodify(&**self.0.as_ref().unwrap(), v)
	}
}
impl<B> OptionalBoxBiwooder<B> where B:?Sized {
	pub const fn new(b:Box<B>)-> Self { OptionalBoxBiwooder(Some(b)) }
	pub const fn empty()-> Self { OptionalBoxBiwooder(None) }
}


pub const fn biwooder_from_fns<W, D>(wf:W, df:D)-> CompositeBiwooder<LambdaWooder<W>, LambdaDewooder<D>>{
	CompositeBiwooder(LambdaWooder(wf), LambdaDewooder(df))
}


pub fn woodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<Wood>)
	where InnerTran: Wooder<T>, I:Iterator<Item=&'a T>, T:'a
{
	for vi in v { output.push(inner.woodify(vi)); }
}
pub fn dewoodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<T>) -> Result<(), DewoodifyError>
	where InnerTran: Dewooder<T>, I:Iterator<Item=&'a Wood>
{
	// let errors = Vec::new();
	for vi in v {
		match inner.dewoodify(vi) {
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


impl<T> Woodable for Vec<T> where T:Woodable {
	fn woodify(&self) -> Wood {
		let mut ret = Vec::new();
		woodify_seq_into(&DefaultBiwooder, self.iter(), &mut ret);
		ret.into()
	}
}
impl<T> Dewoodable for Vec<T> where T:Dewoodable {
	fn dewoodify(v:&Wood) -> Result<Vec<T>, DewoodifyError> {
		let mut ret = Vec::new();
		try!(dewoodify_seq_into(&DefaultBiwooder, v.contents(), &mut ret));
		Ok(ret)
	}
}

#[derive(Copy, Clone)]
pub struct SequenceTran<SubTran>(SubTran);
impl<T, SubTran> Wooder<Vec<T>> for SequenceTran<SubTran> where SubTran:Wooder<T> {
	fn woodify(&self, v:&Vec<T>) -> Wood {
		let mut ret = Vec::new();
		woodify_seq_into(&self.0, v.iter(), &mut ret);
		ret.into()
	}
}
impl<T, SubTran> Dewooder<Vec<T>> for SequenceTran<SubTran> where SubTran:Dewooder<T> {
	fn dewoodify(&self, v:&Wood) -> Result<Vec<T>, DewoodifyError> {
		let mut ret = Vec::new();
		try!(dewoodify_seq_into(&self.0, v.contents(), &mut ret));
		Ok(ret)
	}
}

#[derive(Copy, Clone)]
pub struct TaggedSequenceTran<'a, SubTran>(&'a str, SubTran);
impl<'a, T, SubTran> Wooder<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Wooder<T> {
	fn woodify(&self, v:&Vec<T>) -> Wood {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		woodify_seq_into(&self.1, v.iter(), &mut ret);
		ret.into()
	}
}

fn ensure_tag<'b>(v:&'b Wood, tag:&str) -> Result<std::slice::Iter<'b, Wood>, DewoodifyError> {
	let mut i = v.contents();
	if let Some(name_wood) = i.next() {
		match *name_wood {
			Leafv(ref at)=>{
				let name = at.v.as_str();
				if name == tag {
					Ok(i)
				}else{
					Err(DewoodifyError::new_with_cause(name_wood, format!("expected \"{}\" here, but instead there was \"{}\"", tag, name), None))
				}
			},
			_=> {
				Err(DewoodifyError::new_with_cause(name_wood, format!("expected \"{}\" here, but instead there was a branch wood", tag), None))
			}
		}
	}else{
		Err(DewoodifyError::new_with_cause(v, format!("expected \"{}\" at beginning, but the wood was empty", tag), None))
	}
}

impl<'a, T, SubTran> Dewooder<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Dewooder<T> {
	fn dewoodify(&self, v:&Wood) -> Result<Vec<T>, DewoodifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, &self.0)?;
		dewoodify_seq_into(&self.1, it, &mut ret)?;
		Ok(ret)
	}
}



fn dewoodify_pair<K, V, KeyTran, ValTran>(kt:&KeyTran, vt:&ValTran, v:&Wood) -> Result<(K,V), DewoodifyError>
	where KeyTran:Dewooder<K>, ValTran:Dewooder<V>
{
	match *v {
		Branchv(ref lc)=> {
			if lc.v.len() == 2 {
				unsafe{
					let k = kt.dewoodify(lc.v.get_unchecked(0))?; // safe: we just checked the length
					let v = vt.dewoodify(lc.v.get_unchecked(1))?; //
					Ok((k, v))
				}
			}else{
				Err(DewoodifyError::new_with_cause(v, format!("expected a pair, two elements, but the branch here has {}", lc.v.len()), None))
			}
		}
		Leafv(_)=> {
			Err(DewoodifyError::new_with_cause(v, "expected a pair, but the wood here is an leaf".into(), None))
		}
	}
}

#[derive(Copy, Clone)]
pub struct PairBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Wooder<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Wooder<K>, ValTran:Wooder<V> {
	fn woodify(&self, v:&(K,V)) -> Wood {
		let kt = self.0.woodify(&v.0);
		let vt = self.1.woodify(&v.1);
		branch!(kt, vt).into()
	}
}
impl<K, V, KeyTran, ValTran> Dewooder<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Dewooder<K>, ValTran:Dewooder<V> {
	fn dewoodify(&self, v:&Wood) -> Result<(K,V), DewoodifyError> {
		dewoodify_pair(&self.0, &self.1, v)
	}
}


fn woodify_map<'a, K, V, KeyWooder, ValWooder, I>(ktr:&KeyWooder, vtr:&ValWooder, i:I, o:&mut Vec<Wood>)
	where
		KeyWooder: Wooder<K>,
		ValWooder: Wooder<V>,
		I: Iterator<Item=(&'a K, &'a V)>,
		K: 'a,
		V: 'a,
{
	for (kr, vr) in i {
		o.push(branch!(ktr.woodify(kr), vtr.woodify(vr)).into())
	}
}

fn dewoodify_map<'a, K, V, KeyTran, ValTran, I>(ktr:&KeyTran, vtr:&ValTran, i:I, o:&mut Vec<(K, V)>) -> Result<(), DewoodifyError>
	where
		KeyTran: Dewooder<K>,
		ValTran: Dewooder<V>,
		I: Iterator<Item=&'a Wood>,
{
	for v in i {
		o.push(dewoodify_pair(ktr, vtr, v)?);
	}
	Ok(())
}

impl<K, V> Woodable for HashMap<K, V> where
	K: Eq + Hash + Woodable,
	V: Eq + Hash + Woodable,
{
	fn woodify(&self) -> Wood {
		let mut ret = Vec::new();
		woodify_map(&DefaultWooder, &DefaultWooder, self.iter(), &mut ret);
		ret.into()
	}
}
impl<K, V> Dewoodable for HashMap<K, V>
	where
		K: Eq + Hash + Dewoodable,
		V: Eq + Hash + Dewoodable,
{
	fn dewoodify(v:&Wood) -> Result<HashMap<K,V>, DewoodifyError> {
		let mut ret = Vec::new();
		dewoodify_map(&DefaultDewooder, &DefaultDewooder, v.contents(), &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}

#[derive(Clone)]
pub struct HashMapBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Wooder<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Wooder<K>, ValTran:Wooder<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn woodify(&self, v:&HashMap<K, V>) -> Wood {
		let mut ret = Vec::new();
		woodify_map(&self.0, &self.1, v.iter(), &mut ret);
		ret.into()
	}
}
impl<K, V, KeyTran, ValTran> Dewooder<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Dewooder<K>, ValTran:Dewooder<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn dewoodify(&self, v:&Wood) -> Result<HashMap<K,V>, DewoodifyError> {
		let mut ret = Vec::new();
		dewoodify_map(&self.0, &self.1, v.contents(), &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}

#[derive(Clone)]
pub struct TaggedHashMapBi<'a, KeyTran, ValTran>(&'a str, KeyTran, ValTran);
impl<'a, K, V, KeyTran, ValTran> Wooder<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Wooder<K>, ValTran:Wooder<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn woodify(&self, v:&HashMap<K, V>) -> Wood {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		woodify_map(&self.1, &self.2, v.iter(), &mut ret);
		ret.into()
	}
}
impl<'a, K, V, KeyTran, ValTran> Dewooder<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Dewooder<K>, ValTran:Dewooder<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn dewoodify(&self, v:&Wood) -> Result<HashMap<K,V>, DewoodifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, self.0)?;
		dewoodify_map(&self.1, &self.2, it, &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}




#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn idempotent_int() {
		assert!(90isize == DefaultBiwooder.dewoodify(&DefaultBiwooder.woodify(&90isize)).unwrap());
	}
	
	#[test]
	fn tricky_branch_parse() {
		let brancho = branch!(branch!("tricky", "branch"), branch!("parse"));
		let tranner:SequenceTran<SequenceTran<DefaultBiwooder>> = SequenceTran(SequenceTran(DefaultBiwooder));
		let lv:Vec<Vec<String>> = tranner.dewoodify(&brancho).unwrap();
		assert!(lv.len() == 2);
		assert!(lv[0].len() == 2);
		assert!(lv[0][0].len() == 6);
		assert!(tranner.woodify(&lv).to_string() == "((tricky branch) (parse))");
	}
	
	#[test]
	fn do_hash_map() {
		let t = parse_termpose("a:b c:d d:e e:f").unwrap();
		let bt = TaggedHashMapBi("ob", DefaultBiwooder, DefaultBiwooder);
		let utr: Result<HashMap<String, String>, DewoodifyError> = bt.dewoodify(&t);
		assert!(utr.is_err());
		let tt = parse_termpose("ob a:b c:d d:e e:f").unwrap();
		let hm:HashMap<String, String> = TaggedHashMapBi("ob", DefaultBiwooder, DefaultBiwooder).dewoodify(&tt).unwrap();
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
	fn implicit_biwooders() {
		let t = parse_termpose("a:b c:d").unwrap();
		let exh = give_hm();
		
		let exu:HashMap<String, String> = dewoodify(&t).unwrap();
		
		assert!(exu == exh)
	}
	
	const SPECIAL_SEQ_BIWOODER:SequenceTran<DefaultBiwooder> =
		SequenceTran(DefaultBiwooder);
	#[test]
	fn static_biwooder() {
		let r:Result<Wood, _> = parse_termpose("c c c c c a");
		let v:Vec<char> = SPECIAL_SEQ_BIWOODER.dewoodify(&r.unwrap()).unwrap();
		assert_eq!(v.len(), 6usize);
		assert_eq!(v[5], 'a');
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