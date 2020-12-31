// generate 4xu16 lookup table
let map = new Map();

let index = 0;
for (let first_len = 1; first_len <= 3; first_len++) {
    let first_part = "";
    for (let i = 0; i < Math.min(first_len, 4); i++) {
        first_part += "" + i + ", ";
    }

    for (let i = 0; i < 4-first_len; i++) {
        first_part += "255, "
    }

    for (let second_len = 1; second_len <= Math.min(3, 16-first_len); second_len++) {
        let second_part = "";
        for (let i = 0; i < Math.min(second_len, 4); i++) {
            second_part += "" + (first_len + i) + ", "
        }

        for (let i = 0; i < 4-second_len; i++) {
            second_part += "255, "
        }

        for (let third_len = 1; third_len <= Math.min(3, 16-first_len-second_len); third_len++) {
            let third_part = "";
            for (let i = 0; i < Math.min(third_len, 4); i++) {
                third_part += "" + (first_len + second_len + i) + ", "
            }

            for (let i = 0; i < 4-third_len; i++) {
                third_part += "255, "
            }

            for (let fourth_len = 1; fourth_len <= Math.min(3, 16-first_len-second_len-third_len); fourth_len++) {
                let fourth_part = "";
                for (let i = 0; i < Math.min(fourth_len, 4); i++) {
                    fourth_part += "" + (first_len + second_len + third_len + i) + ", "
                }

                for (let i = 0; i < 4-fourth_len; i++) {
                    fourth_part += "255, "
                }

                console.log(first_part + second_part + third_part + fourth_part, `// ${first_len}, ${second_len}, ${third_len}, ${fourth_len}`);
                map.set([first_len, second_len, third_len, fourth_len].join(","), index);
                index++;
            }
        }
    }
}

console.log()

function ctz(v) {
    var c = 32
    v &= -v
    if (v) c--
    if (v & 0x0000FFFF) c -= 16
    if (v & 0x00FF00FF) c -= 8
    if (v & 0x0F0F0F0F) c -= 4
    if (v & 0x33333333) c -= 2
    if (v & 0x55555555) c -= 1
    return c
}

let table2 = "";
for (let mask = 0; mask < 2**12; mask++) {
    let bm_not = ~mask;
    let first_len_raw = ctz(bm_not);
    let first_len = Math.min(first_len_raw + 1, 3);
    let bm_not_2 = bm_not >> first_len;
    let second_len_raw = ctz(bm_not_2);
    let second_len = Math.min(second_len_raw + 1, 3);
    let bm_not_3 = bm_not_2 >> second_len;
    let third_len_raw = ctz(bm_not_3);
    let third_len = Math.min(third_len_raw + 1, 3);
    let bm_not_4 = bm_not_3 >> third_len;
    let fourth_len_raw = ctz(bm_not_4);
    let fourth_len = Math.min(fourth_len_raw + 1, 3);

    let idx = map.get([first_len, second_len, third_len, fourth_len].join(","));
    if (typeof idx === "undefined") {
        console.log(first_len, second_len, third_len, fourth_len);
    }

    let invalid = (first_len_raw > 3 || second_len_raw > 3 || third_len_raw > 3 || fourth_len_raw > 3) ? 1 : 0;
    let packed = idx | first_len << 8 | second_len << 12 | third_len << 16 | fourth_len << 20 | invalid << 31;

    table2 += `0x${(packed>>>0).toString(16).padStart(8, "0")}, // 0b${mask.toString(2).padStart(16, "0")}\n`;
}
console.log(table2);