use std::marker::PhantomData;
use time::{Duration};
use rand::{StdRng};
use actions::{Creator,Editor,Computor,Testor};

pub struct TestResult<D> {
	pub edits: Vec<Duration>,
	pub full_edits: Vec<Duration>,
	pub computes: Vec<Duration>,
	pub full_computes: Vec<Duration>,
	pd: PhantomData<D>,
}

pub struct EditComputeSequence<C,E,U> {
	pub init: C,
	pub edit: E,
	pub comp: U,
	pub changes: usize,
}
impl<'a,C,E,U,D>
Testor<TestResult<D>>
for EditComputeSequence<C,E,U> where
	C: Creator<Duration,D>,
	E: Editor<Duration,D>,
	U: Computor<Duration,D>,
{
	fn test(&mut self, rng: &mut StdRng) -> TestResult<D> {
		let mut testdata: D;
		let mut edits = Vec::with_capacity(self.changes);
		let mut full_edits = Vec::with_capacity(self.changes);
		let mut computes = Vec::with_capacity(self.changes);
		let mut full_computes = Vec::with_capacity(self.changes);

		// step 1: initialize sequence
		let mut init_result = None;
		let full_init_time = Duration::span(||{
			init_result = Some(self.init.create(rng));
		});
		let (init_time,dat) = init_result.unwrap();
		edits.push(init_time);
		full_edits.push(full_init_time);
		testdata = dat;
		let mut comp_result = None;
		let full_comp_time = Duration::span(||{
			comp_result = Some(self.comp.compute(&testdata,rng));
		});
		let comp_time = comp_result.unwrap();
		computes.push(comp_time);
		full_computes.push(full_comp_time);

		// step 2: run a bunch of edits	
		for _ in 0..self.changes {
			let mut edit_result = None;
			let edit_full_time = Duration::span(||{
				edit_result = Some(self.edit.edit(testdata,rng));
			});
			let (edit_time,dat) = edit_result.unwrap();
			edits.push(edit_time);
			full_edits.push(edit_full_time);
			testdata = dat;
			let mut comp_result = None;
			let full_comp_time = Duration::span(||{
				comp_result = Some(self.comp.compute(&testdata,rng));
			});
			let comp_time = comp_result.unwrap();
			computes.push(comp_time);
			full_computes.push(full_comp_time);
		}

		TestResult {
			edits: edits,
			full_edits: full_edits,
			computes: computes,
			full_computes: full_computes,
			pd: PhantomData,
		}
	}
}