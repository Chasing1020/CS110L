use crossbeam_channel;
use std::{fmt::Display, process, thread, time};

fn parallel_map<T, U, F>(mut input_vec: Vec<T>, num_threads: usize, f: F) -> Vec<U>
where
    F: FnOnce(T) -> U + Send + Copy + 'static,
    T: Send + 'static,
    U: Send + 'static + Default + Display,
{
    let mut output_vec: Vec<U> = Vec::with_capacity(input_vec.len());
    output_vec.resize_with(input_vec.len(), Default::default);
    
    // TODO: implement parallel map!
    let (input_sender, input_receiver) = crossbeam_channel::unbounded();
    let (output_sender, output_receiver) = crossbeam_channel::unbounded();
    let mut threads = vec![];
    for _ in 0..num_cpus::get() {
        let input_receiver = input_receiver.clone();
        let output_sender = output_sender.clone();
        threads.push(thread::spawn(move || {
            while let Ok((i, input)) = input_receiver.recv() {
                output_sender.send((i, f(input))).unwrap();
            }
        }))
    }

    input_vec.drain(..).enumerate().for_each(|(i, v)| {
        input_sender.send((i, v)).unwrap();
    });

    drop(input_sender);
    drop(output_sender);

    while let Ok((i, output)) = output_receiver.recv() {
        output_vec[i] = output;
    }

    for thread in threads {
        thread.join().unwrap();
    }

    output_vec
}

fn main() {

    let v = vec![6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 12, 18, 11, 5, 20];
    let squares = parallel_map(v, 10, |num| {
        println!("{} squared is {}", num, num * num);
        thread::sleep(time::Duration::from_millis(500));
        num * num
    });
    println!("squares: {:?}", squares);
}
