use bbqueue::{BBBuffer, GrantR, GrantW, Consumer, Producer};

fn main(){
	// Create a buffer with six elements
	let bb: BBBuffer<64> = BBBuffer::new();
	let (mut prod, mut cons) = bb.try_split().unwrap();

	// Request space for one byte
	let mut wgr = prod.grant_exact(64).unwrap();

	// Set the data
	wgr[0] = 123;
	wgr[1] = 2;

	assert_eq!(wgr.len(), 64);

	// Make the data ready for consuming
	wgr.commit(30);

	// Read all available bytes
	let mut rgr = cons.read().unwrap();

	//assert_eq!(rgr[0], 123);
	println!("length = {:?} {:?}", rgr.len(), rgr[1]);

	// Release the space for later writes
	rgr.release(64);
	// //(prod, cons) = bb.try_split().unwrap();
	wgr = prod.grant_exact(30).unwrap();
	// wgr[0] = 121;
	// wgr[1] = 4;
	wgr.commit(30);

	rgr = cons.read().unwrap();

	assert_eq!(rgr[0], 123);
	println!("length = {:?} {:?}", rgr.len(), rgr[1]);
}