#[macro_export]
macro_rules! matches {
  ($expression:expr, $($pattern:tt)+) => {
    match $expression {
      $($pattern)+ => true,
      _ => false
    }
  }
}

#[macro_export]
macro_rules! matches_opt {
  ($expression:expr, $(|)* $pattern:pat $(|$pattern_extra:pat)* $(if $ifguard:expr)* => $result:expr) => {
    match $expression {
      $pattern $(|$pattern_extra)* $(if $ifguard)* => Some($result),
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! closet {
  (@as_expr $e:expr) => { $e };

  ([$($var:ident),*] $cl:expr) => {{
    $(let $var = $var.clone();)*
      $cl
  }};
}
