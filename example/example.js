function foo(a) {
  let b,c,d;
  if (a == 10) {
    if (a+10 < 30) {
      b = 200 + 100 === 10;
      b = false + true;
    } else {
      b = "hello";
      let b = 1+2;
      b = "world";
    }
  } else {
    b = null;
  }
  b = {kk:null, nn:555n};
  return b;
}

const v = foo(undefined);
