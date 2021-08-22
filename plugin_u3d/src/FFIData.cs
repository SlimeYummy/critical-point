using System;
using System.Collections;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

namespace CriticalPoint {
    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec2f {
        public float x;
        public float y;

        public FFIVec2f(float x, float y) {
            this.x = x;
            this.y = y;
        }

        public static explicit operator FFIVec2f(Vector2 v) {
            return new FFIVec2f(v.x, v.y);
        }

        public static explicit operator Vector2(FFIVec2f v) {
            return new Vector2(v.x, v.y);
        }

        public override string ToString() {
            return string.Format("FFIVec2f({0}, {1})", this.x, this.y);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec2i {
        public int x;
        public int y;

        public FFIVec2i(int x, int y) {
            this.x = x;
            this.y = y;
        }

        public static explicit operator FFIVec2i(Vector2Int v) {
            return new FFIVec2i(v.x, v.y);
        }

        public static explicit operator Vector2Int(FFIVec2i v) {
            return new Vector2Int(v.x, v.y);
        }

        public override string ToString() {
            return string.Format("FFIVec2i({0}, {1})", this.x, this.y);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec3f {
        public float x;
        public float y;
        public float z;

        public FFIVec3f(float x, float y, float z) {
            this.x = x;
            this.y = y;
            this.z = z;
        }

        public static explicit operator FFIVec3f(Vector3 v) {
            return new FFIVec3f(v.x, v.y, v.z);
        }

        public static explicit operator Vector3(FFIVec3f v) {
            return new Vector3(v.x, v.y, v.z);
        }

        public override string ToString() {
            return string.Format("FFIVec3f({0}, {1}, {2})", this.x, this.y, this.z);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec3i {
        public int x;
        public int y;
        public int z;

        public FFIVec3i(int x, int y, int z) {
            this.x = x;
            this.y = y;
            this.z = z;
        }

        public static explicit operator FFIVec3i(Vector3Int v) {
            return new FFIVec3i(v.x, v.y, v.z);
        }

        public static explicit operator Vector3Int(FFIVec3i v) {
            return new Vector3Int(v.x, v.y, v.z);
        }

        public override string ToString() {
            return string.Format("FFIVec3i({0}, {1}, {2})", this.x, this.y, this.z);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec4f {
        public float x;
        public float y;
        public float z;
        public float w;

        public FFIVec4f(float x, float y, float z, float w) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        public static explicit operator FFIVec4f(Vector4 v) {
            return new FFIVec4f(v.x, v.y, v.z, v.w);
        }

        public static explicit operator Vector4(FFIVec4f v) {
            return new Vector4(v.x, v.y, v.z, v.w);
        }

        public override string ToString() {
            return string.Format("FFIVec4f({0}, {1}, {2}, {3})", this.x, this.y, this.z, this.w);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIVec4i {
        public int x;
        public int y;
        public int z;
        public int w;

        public FFIVec4i(int x, int y, int z, int w) {
            this.x = x;
            this.y = y;
            this.z = z;
            this.w = w;
        }

        public override string ToString() {
            return string.Format("FFIVec4i({0}, {1}, {2}, {3})", this.x, this.y, this.z, this.w);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIComplex {
        public float re;
        public float im;

        public FFIComplex(float re, float im) {
            this.re = re;
            this.im = im;
        }

        public override string ToString() {
            return string.Format("FFIComplex({0}, {1}", this.re, this.im);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIQuaternion {
        public float i;
        public float j;
        public float k;
        public float w;

        public FFIQuaternion(float w, float i, float j, float k) {
            this.i = i;
            this.j = j;
            this.k = k;
            this.w = w;
        }

        public static explicit operator FFIQuaternion(Quaternion v) {
            return new FFIQuaternion(v.w, v.x, v.y, v.z);
        }

        public static explicit operator Quaternion(FFIQuaternion v) {
            return new Quaternion(v.i, v.j, v.k, v.w);
        }

        public override string ToString() {
            return string.Format("FFIQuaternion({0}, {1}, {2}, {3})", this.w, this.i, this.j, this.k);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIMat2f {
        public float m11;
        public float m12;
        public float m21;
        public float m22;

        public FFIMat2f(
            float m11,
            float m12,
            float m21,
            float m22
        ) {
            this.m11 = m11;
            this.m12 = m12;
            this.m21 = m21;
            this.m22 = m22;
        }

        public override string ToString() {
            return string.Format("FFIMat2f({0}, {1}, {2}, {3})", this.m11, this.m12, this.m21, this.m22);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIMat3f {
        public float m11;
        public float m12;
        public float m13;
        public float m21;
        public float m22;
        public float m23;
        public float m31;
        public float m32;
        public float m33;

        public FFIMat3f(
            float m11,
            float m12,
            float m13,
            float m21,
            float m22,
            float m23,
            float m31,
            float m32,
            float m33
        ) {
            this.m11 = m11;
            this.m12 = m12;
            this.m13 = m13;
            this.m21 = m21;
            this.m22 = m22;
            this.m23 = m23;
            this.m31 = m31;
            this.m32 = m32;
            this.m33 = m33;
        }

        public override string ToString() {
            return string.Format(
                "FFIMat3f({0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}, {8})",
                this.m11, this.m12, this.m13,
                this.m21, this.m22, this.m23,
                this.m31, this.m32, this.m33
            );
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIMat4f {
        public float m11;
        public float m12;
        public float m13;
        public float m14;
        public float m21;
        public float m22;
        public float m23;
        public float m24;
        public float m31;
        public float m32;
        public float m33;
        public float m34;
        public float m41;
        public float m42;
        public float m43;
        public float m44;

        public FFIMat4f(
            float m11,
            float m12,
            float m13,
            float m14,
            float m21,
            float m22,
            float m23,
            float m24,
            float m31,
            float m32,
            float m33,
            float m34,
            float m41,
            float m42,
            float m43,
            float m44
        ) {
            this.m11 = m11;
            this.m12 = m12;
            this.m13 = m13;
            this.m14 = m14;
            this.m21 = m21;
            this.m22 = m22;
            this.m23 = m23;
            this.m24 = m24;
            this.m31 = m31;
            this.m32 = m32;
            this.m33 = m33;
            this.m34 = m34;
            this.m41 = m41;
            this.m42 = m42;
            this.m43 = m43;
            this.m44 = m44;
        }

        public static explicit operator FFIMat4f(Matrix4x4 v) {
            return new FFIMat4f(
                v.m00,
                v.m01,
                v.m02,
                v.m03,
                v.m10,
                v.m11,
                v.m12,
                v.m13,
                v.m20,
                v.m21,
                v.m22,
                v.m23,
                v.m30,
                v.m31,
                v.m32,
                v.m33
            );
        }

        public static explicit operator Matrix4x4(FFIMat4f v) {
            Matrix4x4 m = Matrix4x4.zero;
            m.m00 = v.m11;
            m.m01 = v.m12;
            m.m02 = v.m13;
            m.m03 = v.m14;
            m.m10 = v.m21;
            m.m11 = v.m22;
            m.m12 = v.m23;
            m.m13 = v.m24;
            m.m20 = v.m31;
            m.m21 = v.m32;
            m.m22 = v.m33;
            m.m23 = v.m34;
            m.m30 = v.m41;
            m.m31 = v.m42;
            m.m32 = v.m43;
            m.m33 = v.m44;
            return m;
        }

        public override string ToString() {
            return string.Format(
                "FFIMat3f({0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}, {8}, {9}, {10}, {11}, {12}, {13}, {14}, {15})",
                this.m11, this.m12, this.m13, this.m14,
                this.m21, this.m22, this.m23, this.m24,
                this.m31, this.m32, this.m33, this.m34,
                this.m41, this.m42, this.m43, this.m44
            );
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFIArray<T> : IEnumerable<T> where T : unmanaged {
#if UNITY_64
        private readonly IntPtr ptr;
        public readonly ulong cap;
        public readonly ulong len;

        public ref T this[ulong idx] {
            get {
                unsafe {
                    return ref *((T*)this.ptr + idx);
                }
            }
        }

        public ref T this[int idx] {
            get {
                unsafe {
                    return ref *((T*)this.ptr + (ulong)idx);
                }
            }
        }
#else
        public IntPtr ptr;
        public uint cap;
        public uint len;
        
        public ref T this[uint idx] {
            get {
                unsafe {
                    return ref *((T*)this.ptr + idx);
                }
            }
        }

        public ref T this[int idx] {
            get {
                unsafe {
                    return ref *((T*)this.ptr + (int)idx);
                }
            }
        }
#endif
        public List<T> ToList() {
            List<T> list = new List<T>();
            list.Capacity = (int)this.len;
            for (ulong idx = 0; idx < this.len; ++idx) {
                unsafe {
                    list[(int)idx] = *((T*)this.ptr + idx);
                };
            }
            return list;
        }

        public IEnumerator<T> GetEnumerator() {
            for (ulong idx = 0; idx < this.len; ++idx) {
                yield return this[idx];
            }
        }

        IEnumerator IEnumerable.GetEnumerator() {
            return this.GetEnumerator();
        }

        public override string ToString() {
            return string.Format("FFIArray(ptr: 0x{0:X}, len: {1}, cap: {2})", this.ptr, this.len, this.cap);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FFISlice<T> where T : unmanaged {
#if UNITY_64
        public IntPtr ptr;
        public ulong len;
#else
        public IntPtr ptr;
        public uint len;
#endif

        public override string ToString() {
            return string.Format("FFIArray(ptr: 0x{0:X}, len: {1})", this.ptr, this.len);
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct ObjID : IEqualityComparer<ObjID> {
        public ulong id;

        public static readonly ObjID Invalid = new ObjID(0xFFFF_FFFF_FFFF_FFFF);

        public ObjID(ulong id) {
            this.id = id;
        }

        public bool IsValid() {
            return this.id < Invalid.id;
        }

        public bool IsInvalid() {
            return this.id >= Invalid.id;
        }

        public bool Equals(ObjID a, ObjID b) {
            return a.id == b.id;
        }

        public int GetHashCode(ObjID a) {
            return this.id.GetHashCode();
        }

        public override bool Equals(object obj) {
            if (this.GetType() != obj.GetType()) {
                return false;
            }
            return Equals(this, (ObjID)obj);
        }

        public override int GetHashCode() {
            return this.id.GetHashCode();
        }

        public override string ToString() {
            return string.Format("ObjID({0})", this.id);
        }

        public static bool operator ==(ObjID a, ObjID b) {
            return a.id == b.id;
        }

        public static bool operator !=(ObjID a, ObjID b) {
            return a.id != b.id;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct FastResID : IEqualityComparer<FastResID> {
        public ulong id;

        public FastResID(ulong id) {
            this.id = id;
        }

        public bool Equals(FastResID a, FastResID b) {
            return a.id == b.id;
        }

        public int GetHashCode(FastResID a) {
            return this.id.GetHashCode();
        }

        public override bool Equals(object obj) {
            if (this.GetType() != obj.GetType()) {
                return false;
            }
            return Equals(this, (FastResID)obj);
        }

        public override int GetHashCode() {
            return this.id.GetHashCode();
        }

        public override string ToString() {
            return string.Format("FastResID({0})", this.id);
        }

        public static bool operator ==(FastResID a, FastResID b) {
            return a.id == b.id;
        }

        public static bool operator !=(FastResID a, FastResID b) {
            return a.id != b.id;
        }
    }

    public enum OptAction : byte {
        None,
        Move,
        Dash,
        Defend,
        DefendEx,
        Attack1,
        Attack1Ex,
        Attack2,
        Attack2Ex,
        Skill1,
        Skill1Ex,
        Skill2,
        Skill2Ex,
        Skill3,
        Skill3Ex,
        Item1,
        Item2,
        Item3,
        Interact,
        Lock,
    }

    public enum OptState : byte {
        Start,
        Finish,
    }

    [StructLayout(LayoutKind.Sequential)]
    struct Operation {
        public OptAction action;
        public OptState state;
        public FFIVec2f direction;

        public Operation(OptAction action, OptState state) {
            this.action = action;
            this.state = state;
            this.direction = new FFIVec2f();
        }

        public Operation(OptAction action, OptState state, FFIVec2f dir) {
            this.action = action;
            this.state = state;
            this.direction = dir;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    struct LogicPropHeader {
        public ObjID objId;
        public ClassID classId;
        private IntPtr vtable;
        private long _padding_; // for 16bit align

        public static readonly int Size = Marshal.SizeOf<LogicPropHeader>();

        public unsafe static ref LogicPropHeader FromPtr(IntPtr ptr) {
            return ref *(LogicPropHeader*)ptr;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LogicProp<P> where P : unmanaged {
        public ObjID objId;
        public ClassID classId;
        private IntPtr vtable;
        private long _padding_; // for 16bit align
        public P prop;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct LogicStateHeader {
        public ObjID objId;
        public ClassID classId;
        public LogicLifecycle lifecycle;
        private IntPtr vtable;
        private long _padding_; // for 16bit align

        public static readonly int Size = Marshal.SizeOf<LogicPropHeader>();

        public unsafe static ref LogicStateHeader FromPtr(IntPtr ptr) {
            return ref *(LogicStateHeader*)ptr;
        }
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct LogicState<S> where S : unmanaged {
        public ObjID objId;
        public ClassID classId;
        public LogicLifecycle lifecycle;
        private IntPtr vtable;
        private long _padding_; // for 16bit align
        public S state;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct DataPool {
        public FFIArray<IntPtr> props;
        public FFIArray<IntPtr> states;
    }
}
