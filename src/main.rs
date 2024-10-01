use ndarray::Array2;
use ndarray::{s, ArrayBase, arr2};
use core::f32;
use std::str::FromStr;
use csv::{ReaderBuilder, StringRecord};
use std::io;
use jack;
use num_traits;
use std::fmt::Debug;

fn read_csv<U>(path: String, rows: usize, columns: usize) -> Array2<U> 
where U: std::clone::Clone, U: num_traits::identities::Zero, U: std::str::FromStr, <U as FromStr>::Err: Debug{ 
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(path).unwrap();
    let mut str: StringRecord;
    let mut arr: Array2<U> = ndarray::ArrayBase::zeros([rows, columns]);
    let mut idx: usize = 0;
    for result in rdr.records() {
        str = result.unwrap();
        for i in 0..str.len() {
            let var = str.get(i).unwrap().parse::<U>().unwrap();
            arr[[idx, i]] = var;
        }
        idx += 1;
    }
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        let path = String::from("test.csv");
        assert!(read_csv::<f32>(path, 2, 3) == arr2(&[[1.0, 2.0, 3.0],
                                         [4.0, 5.0, 6.0]]));
    }
}

fn main() {
    let pos = read_csv::<isize>(String::from("pos_4096.csv"), 4096, 3);
    let hrir_mp = read_csv::<f32>(String::from("hrir_mp_4096.csv"), 4096, 784);
    let itd = read_csv::<f32>(String::from("itd_filtered_4096.csv"), 1, 4096);


    let (client, _status) = jack::Client::new("blub", jack::ClientOptions::NO_START_SERVER).unwrap();
    let mut output = client.register_port("out", jack::AudioOut::default()).unwrap();
    let sample_rate = 48000;

    let az = 45;
    let el = 0;
    let mut el_idx: Vec<usize> = vec![];
    let mut final_idx: usize = 0;
    let el_slice: ArrayBase<ndarray::ViewRepr<&isize>, ndarray::Dim<[usize; 1]>>  = pos.slice(s![.., 1]);
    let az_slice: ArrayBase<ndarray::ViewRepr<&isize>, ndarray::Dim<[usize; 1]>> = pos.slice(s![.., 0]);
    let mut found_el = false;
    let mut found_az = false;


    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let out_p = output.as_mut_slice(ps);
        
        for i in 0..el_slice.len() {
            if el_slice[i as usize] == el {
                el_idx.push(i);
                found_el = true;
            }   else if found_el == true {break};          
        }

        if found_el == false {
            println!("elevation not found");
        }
    
        for i in el_idx.iter() {
            if az_slice[*i as usize] == az {
                final_idx = *i as usize;
                found_az = true;
                break
            }
        }
    
        if found_az == false {
            println!("elevation not found");
        }
    
        let hrir: Vec<f32> = hrir_mp.slice_move(s![final_idx, ..]).to_vec();
        let hrir_r: Vec<f32> = hrir[0..384].to_vec();
        let hrir_l: Vec<f32> = hrir[384..768].to_vec();
    
        let delay: isize = itd[[0, final_idx]].round() as isize;


        out_p.iter_mut().for_each(|o| {

        });

        jack::Control::Continue
    };
    println!("Press Enter to quit!");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
}