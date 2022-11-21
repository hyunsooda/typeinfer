function foo(a) {
  let b,c,d;
  if (a == 10) {
    if (a+10 < 30) {
      b = 200 + 100 === 10;
    } else {
      b = "hello";
    }
  } else {
    b = 100;
  }
  return b;
}

const v = foo(100);
console.log(typeof v);
