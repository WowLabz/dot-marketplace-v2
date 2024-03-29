// Random shuffling using Fischer-Yates modern method & Linear Congruential Generator

use parity_scale_codec::alloc::string::ToString;
use sp_std::vec::Vec;
#[allow(unused_imports)]
use num_traits::float::Float;

pub fn dot_shuffle<T>(
    mut input: Vec<T>, 
    seed: u32, 
    mut length: u32
) -> Vec<T> {
    // ----- Initializations
    let mut random_number_idx: u32;
    let mut random_element: T;
    let mut result: Vec<T> = Vec::new();
    // -----
    
    // ----- Executing the Fischer-Yates algorithm
    while !input.is_empty() {
        // * Function for adding individual numbers
        random_number_idx = get_sum(
            // * Linear congruential generator
            lcg(
                seed as u128, 
                length as u128
            ), 
            length
        );
        // * Index should be less than the length of the vector
        if random_number_idx > input.len() as u32 - 1 {
            random_number_idx -= 1;
        }
        // * As input will decrease along with it the length should too
        length -= 1;
        // The removed element is swaped with the last element
        random_element = input.swap_remove(random_number_idx as usize);
        // Updating the vector
        result.push(random_element);
    }
    // -----
    
    result
}

pub fn get_sum(number: u32, length: u32) -> u32 {
    // Converting numeric to string
    let mut number_string = number.to_string();
    // For total of the number
    let mut sum_of_number: u32;
    
    // ----- Splitting the number and adding all the individual numbers
    loop {
        // * Converting string -> char -> digit
        sum_of_number = number_string.chars().map(|d| d.to_digit(10).unwrap()).sum();
        if sum_of_number <= length {
            break;
        } else {
            if sum_of_number > length {
                sum_of_number -= 1;
            }
            number_string = sum_of_number.to_string();
        }
    }
    // -----
    
    sum_of_number
}

pub fn lcg(seed: u128, length: u128) -> u32 {
    // ----- Initializations
    let mut x;
    let mut random_numbers = Vec::new();
    let mut total: u128 = 0;
    // -----
    
    // Multiplier
    const A: u128 = 5;
    // Increment
    const B: u128 = 2;
    // Modulus
    let m: u128 = seed + 10;
    
    // ----- Calculation
    for i in 0..length {
        x = (A * i + B) % m;
        random_numbers.push(x);
        total += x / 2;
    }
    // -----

    total as u32
}

pub fn roundoff(total_rating: u8, number_of_users: u8) -> u8 {
    // For carrying the result
    let output: u8;
    // Calculating the average rating in floating point value
    let avg_rating: f32 = total_rating as f32 / number_of_users as f32;
    // Converting floating point to integer
    let rounded_avg_rating: u8 = avg_rating as u8;
    // Removing the decimal from float
    let fraction = avg_rating.fract();

    // ----- Result at different conditions
    if rounded_avg_rating != 0 {
        if fraction >= 0.5 {
            output = rounded_avg_rating + 1;
        } else {
            output = rounded_avg_rating; 
        }
    } else {
        output = 0;
    }
    // -----

    output
}
