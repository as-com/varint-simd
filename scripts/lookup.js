// generate 2xu32 lookup table
let map = new Map();

let index = 0;
for (let first_len = 1; first_len <= 10; first_len++) {
    let first_part = "";
    for (let i = 0; i < Math.min(first_len, 8); i++) {
        first_part += "" + i + ", ";
    }

    for (let i = 0; i < 8-first_len; i++) {
        first_part += "0, "
    }

    for (let second_len = 1; second_len <= 10; second_len++) {
        let second_part = "";
        for (let i = 0; i < Math.min(second_len, 8); i++) {
            second_part += "" + (first_len + i) + ", "
        }

        for (let i = 0; i < 8-second_len; i++) {
            second_part += "0, "
        }

        console.log(first_part + second_part, `// ${first_len}, ${second_len}`);
        map.set([first_len, second_len].join(","), index);
        index++;
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
let table3 = "";
for (let mask = 0; mask < 2**10; mask++) {
    let bm_not = ~mask;
    let first_len = ctz(bm_not) + 1;
    let bm_not_2 = bm_not >> first_len;
    let second_len = ctz(bm_not_2) + 1;

    let idx = map.get([first_len, second_len].join(","));
    if (typeof idx === "undefined") {
        console.log(first_len, second_len);
    }

    table2 += `(${idx}, ${first_len}, ${second_len}), // 0b${mask.toString(2).padStart(16, "0")}\n`;
}
console.log(table2);