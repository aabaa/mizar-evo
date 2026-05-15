# Sample Codes

> Canonical language: English. Japanese companion: [../ja/sample_codes.md](../ja/sample_codes.md).

These examples are illustrative library sketches. They are intended to show how
the surface syntax from the specification fits together; omitted proofs are
written as `...`, and omitted correctness obligations are shown only by name.

## Directory structure

```bash
mml/
    function/
    algebra/
        structure/
    number/
        structure/
```

## /mml/function

- function.miz

    ```mizar
    definition
      let X,Y be set;
      mode FunctionDef: Function of X,Y is quasi_total PartFunc of X,Y;
    end;
    ```

- binop.miz

    ```mizar
    import .function;
    import mml.algebra.structure.sorted;

    definition
      let S be 1-sorted;
      mode BinOpDef: BinOp of S is Function of [: S.carrier, S.carrier :], S.carrier;
    end;
    ```

## /mml/algebra/structure

- sorted.miz

    ```mizar
    import mml.function.function;

    definition
      :: This grammar rule is specialized for set.
      struct 1-sorted where
        field carrier -> set;
      end;

      :: Inherit all functors and attributes from set.
      :: `it` means `set` itself.
      :: Sometimes a `field ... from ...` syntax requires type conversion.
      :: In that case, prove the consistency of the type conversion
      :: with a `coherence` block.
      :: `cluster` statements might be useful.
      inherit 1-sorted extends set where
        field carrier from it;
      end;
    end;

    definition
      struct UnitStr where
        field carrier -> set;
        property unit -> Element of carrier;
      end;

      :: If some fields or properties of the base structure are not inherited,
      :: the Mizar analyzer will give an error message.
      inherit UnitStr extends 1-sorted;
    end;

    definition
      struct ZeroStr where
        field carrier -> set;
        property zero -> Element of carrier;
      end;

      inherit ZeroStr extends UnitStr where
        field carrier from carrier;
        property zero from unit;    :: renamed
      end;
    end;

    definition
      struct OneStr where
        field carrier -> set;
        property one -> Element of carrier;
      end;

      inherit OneStr extends UnitStr where
        field carrier from carrier;
        property one from unit;     :: renamed
      end;
    end;

    definition
      struct ZeroOneStr where
        field carrier -> set;
        property zero -> Element of carrier;
        property one -> Element of carrier;
      end;

      inherit ZeroOneStr extends ZeroStr;
      inherit ZeroOneStr extends OneStr;
    end;

    definition
      struct 2-sorted where
        field carrier -> set;
        field carrier' -> set;
      end;

      inherit 2-sorted extends 1-sorted;
    end;
    ```

- magma.miz

    ```mizar
    import .sorted;

    definition
      struct Magma where
        field carrier -> set;
        field binop -> BinOp of carrier;
      end;

      inherit Magma extends 1-sorted;
    end;

    definition
      struct AddMagma where
        field carrier -> set;
        field add -> BinOp of carrier;
      end;

      inherit AddMagma extends Magma where
        field carrier from carrier;
        field add from binop;       :: renamed
      end;
    end;

    definition
      struct MulMagma where
        field carrier -> set;
        field mul -> BinOp of carrier;
      end;

      inherit MulMagma extends Magma where
        field carrier from carrier;
        field mul from binop;       :: renamed
      end;
    end;

    ```

- loopstr.miz

    ```mizar
    import .magma;

    definition
      struct LoopStr where
        field carrier -> set;
        field binop -> BinOp of carrier;
        property unit -> Element of carrier;
      end;

      inherit LoopStr extends Magma;
      
      ::=
        The Mizar analyzer must check the consistency for diamond inheritance.
        It means the both following paths introduce the same fields or properties:
        Path 1: AddLoopStr.add -> LoopStr.binop -> Magma.binop
        Path 2: AddLoopStr.add -> AddMagma.add -> Magma.binop
      =::
      struct AddLoopStr where
        field carrier -> set;
        field add -> BinOp of carrier;
        property zero -> Element of carrier;
      end;

      inherit AddLoopStr extends LoopStr where
        field carrier from carrier;
        field add from binop;       :: renamed
        property zero from unit;    :: renamed
      end;

      inherit AddLoopStr extends AddMagma;

      struct MulLoopStr where
        field carrier -> set;
        field mul -> BinOp of carrier;
        property one -> Element of carrier;
      end;

      inherit MulLoopStr extends LoopStr where
        field carrier from carrier;
        field mul from binop;       :: renamed
        property one from unit;     :: renamed
      end;

      inherit MulLoopStr extends MulMagma;

      struct DoubleLoopStr where
        field carrier -> set;
        field add -> BinOp of carrier;
        field mul -> BinOp of carrier;
        property zero -> Element of carrier;
        property one -> Element of carrier;
      end;

      inherit DoubleLoopStr extends AddLoopStr;
      inherit DoubleLoopStr extends MulLoopStr;
    end;

    definition
      let A be AddLoopStr;
      let x,y be Element of A.carrier;

      @latex("x+y")
      func AddDef: x + y -> Element of A.carrier equals A.add(x,y);
    end;

    infix_operator("+", left, 80);

    definition
      let M be MulLoopStr;
      let x,y be Element of M.carrier;

      @latex("x\\cdot y")
      func MulDef: x * y -> Element of M.carrier equals M.mul(x,y);
    end;

    infix_operator("*", left, 90);

    ```

- group.miz

    ```mizar
    import .loopstr;

    definition
      let M be Magma;

      attr M is associative means
        for x,y,z being Element of M holds
        M.binop(M.binop(x,y),z) = M.binop(x,M.binop(y,z));

      attr M is unital means
        ex e being Element of M st
        for x being Element of M holds
        M.binop(x,e) = x & M.binop(e,x) = x;

      func id. M -> Element of unital M means
        for x being Element of M holds
        M.binop(x, it) = x & M.binop(it, x) = x;
      existence;
      uniqueness;

      attr M is commutative means
        for x,y being Element of M holds
        M.binop(x,y) = M.binop(y,x);
    end;

    definition
      let M be LoopStr;

      redefine attr M is unital means
        (M qua Magma) is unital & M.unit = id. M;

      attr M is invertible means
        for x being Element of M
        ex y being Element of M
        st M.binop(x,y) = M.unit
         & M.binop(y,x) = M.unit;

      mode GroupDef: Group is non empty associative invertible unital LoopStr;
    end;
    ```

- ring.miz

    ```mizar
    import .group;

    definition
      let R be DoubleLoopStr;

      attr RingLikeDef: R is ring_like means
        (R qua AddLoopStr) is commutative Group &
        (R qua MulLoopStr) is associative unital &
        R.zero <> R.one &
        for x,y,z being Element of R holds
        x * (y+z) = x*y + x*z;

      mode RingDef: Ring is ring_like DoubleLoopStr;
    end;

    definition
      let R be Ring;

      ::=
        If `R is commutative` is stated without declaring the following `attr`,
        the analyzer returns an error because it cannot determine
        whether the `commutative` is associated with `AddLoopStr` or `MulLoopStr`.
      =::
      attr R is commutative means
      (R qua MulLoopStr) is commutative;
    end;
    ```

- field.miz
- module.miz
- vector.miz

## /mml/number/structure

- natural.miz
- integer.miz
- rational.miz
- real.miz
- complex.miz

## Annotation and proof-development snippets

The following snippets illustrate the annotation forms summarized in
[Appendix E](./appendix_e.annotation_quick_reference.md).

```mizar
definition
  let a,b be Nat;

  @latex("\\gcd(a,b)")
  func GcdDef: Gcd(a,b) -> Nat means ...;

  ::: Computes the greatest common divisor of two positive natural numbers.
  :::
  ::: @param a  the first input
  ::: @param b  the second input
  ::: @returns  the greatest common divisor of `a` and `b`
  ::: @requires `a > 0 & b > 0`
  terminating algorithm euclid_gcd(a, b) -> Nat
    requires a > 0 & b > 0
    ensures result = Gcd(a, b)
    decreasing a + b
  do
    ...
  end;
end;

theorem Gcd_commutes:
  for a,b being Nat st a > 0 & b > 0 holds
    Gcd(a,b) = Gcd(b,a)
proof
  @show_thesis
  @proof_hint(max_axioms: 32, solver: vampire)
  thus thesis by Gcd_def, Euclid_step;
end;

@show_type(euclid_gcd(48, 18))
@eval(euclid_gcd(48, 18))
```
