using System;
using System.Linq;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using UnityEngine;

namespace CriticalPoint {
    internal static class Sys {
        [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
        static public extern IntPtr LoadLibrary(string fileName);

        [DllImport("kernel32", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static public extern bool FreeLibrary(IntPtr hModule);

        [DllImport("kernel32")]
        static public extern IntPtr GetProcAddress(IntPtr hModule, string funcName);
    }

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    [return: MarshalAs(UnmanagedType.Bool)]
    internal delegate bool InitLoggerFunc(
        [MarshalAs(UnmanagedType.LPStr)] string logPath
    );

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate IntPtr CreateResCacheFunc(
        [MarshalAs(UnmanagedType.LPStr)] string rootPath,
        [MarshalAs(UnmanagedType.LPStr)] string resFile,
        [MarshalAs(UnmanagedType.LPStr)] string idFile
    );

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate void DestoryResCacheFunc(IntPtr resCache);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate IntPtr CreateSyncAgentFunc(
        IntPtr resCache,
        uint fps,
        [MarshalAs(UnmanagedType.LPStr)] string initId
    );

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate void DestorySyncAgentFunc(IntPtr syncAgent);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate IntPtr SyncAgentUpdateFunc(IntPtr syncAgent);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    internal delegate void FreeDataPoolFunc(IntPtr dataPool);

    internal static class FFI {
        private static IntPtr hModule;

        public static InitLoggerFunc InitLogger;
        public static CreateResCacheFunc CreateResCache;
        public static DestoryResCacheFunc DestoryResCache;
        public static CreateSyncAgentFunc CreateSyncAgent;
        public static DestorySyncAgentFunc DestorySyncAgent;
        public static SyncAgentUpdateFunc SyncAgentUpdate;
        public static FreeDataPoolFunc FreeDataPool;

        public static void Load() {
            if (hModule != IntPtr.Zero) {
                throw new Exception("Call CriticalPoint::LoadFunc() twice");
            }

            hModule = Sys.LoadLibrary("./Assets/CriticalPoint/critical_point_u3d.dll");
            if (hModule == IntPtr.Zero) {
                throw new Exception("Load critical_point_u3d.dll failed");
            }

            InitLogger = LoadFunc<InitLoggerFunc>("init_logger");
            CreateResCache = LoadFunc<CreateResCacheFunc>("create_res_cache");
            DestoryResCache = LoadFunc<DestoryResCacheFunc>("destroy_res_cache");
            CreateSyncAgent = LoadFunc<CreateSyncAgentFunc>("create_sync_agent");
            DestorySyncAgent = LoadFunc<DestorySyncAgentFunc>("destroy_sync_agent");
            SyncAgentUpdate = LoadFunc<SyncAgentUpdateFunc>("sync_agent_update");
            FreeDataPool = LoadFunc<FreeDataPoolFunc>("free_data_pool");
        }

        private static F LoadFunc<F>(string funcName) where F : Delegate {
            IntPtr pFunc = Sys.GetProcAddress(hModule, funcName);
            if (pFunc == IntPtr.Zero) {
                throw new Exception("Method not found " + funcName + "()");
            }
            return Marshal.GetDelegateForFunctionPointer<F>(pFunc);
        }

        public static void Unload() {
            if (hModule != IntPtr.Zero) {
                Sys.FreeLibrary(hModule);
                hModule = IntPtr.Zero;
            }
        }
    }

    [AttributeUsage(AttributeTargets.Struct, AllowMultiple = false, Inherited = false)]
    internal class DefProp : Attribute {
        public ClassID classId { get; }

        public DefProp(ClassID classId) {
            this.classId = classId;
        }
    }

    [AttributeUsage(AttributeTargets.Struct, AllowMultiple = false, Inherited = false)]
    internal class DefState : Attribute {
        public ClassID classId { get; }

        public DefState(ClassID classId) {
            this.classId = classId;
        }
    }

    public abstract class BaseRefState {
        internal BaseRefState next;
        protected ObjID _objId;
        protected ClassID _classId;
        protected IntPtr _state;

        public ClassID classId { get { return _classId; } }

        public ObjID objId { get { return _objId; } }

        internal void SetPtr(IntPtr ptr) {
            this._state = ptr;
        }
    }

    public class RefState<S> : BaseRefState where S : unmanaged {
        public RefState(ObjID objId) {
            var attrs = Attribute.GetCustomAttributes(typeof(S));
            if (attrs.Length == 0) {
                throw new Exception("Invalid Prop type");
            }
            var attr = (DefState)attrs.FirstOrDefault();

            this.next = null;
            this._objId = objId;
            this._classId = attr.classId;
            this._state = IntPtr.Zero;

            BaseRefState state;
            if (SyncAgent.states.TryGetValue(this.objId, out state)) {
                this.next = state;
            }
            SyncAgent.states[this.objId] = this;
        }

        ~RefState() {
            this._classId = ClassID.Invalid;
            this._objId = ObjID.Invalid;
            this._state = IntPtr.Zero;
        }

        void Free() {
            this._classId = ClassID.Invalid;
            this._objId = ObjID.Invalid;
            this._state = IntPtr.Zero;
        }

        public bool IsValid() {
            return this._state != IntPtr.Zero;
        }

        public bool IsInvalid() {
            return this._state == IntPtr.Zero;
        }

        public LogicLifecycle lifecycle {
            get {
                if (this.IsInvalid()) {
                    throw new Exception("Visit null state");
                }
                unsafe {
                    return ((LogicState<S>*)this._state)->lifecycle;
                };
            }
        }

        public ref S state {
            get {
                if (this.IsInvalid()) {
                    throw new Exception("Visit null state");
                }
                unsafe {
                    return ref ((LogicState<S>*)this._state)->state;
                };
            }
        }

        public override string ToString() {
            return string.Format("RefState(classId: {0}, objId: {1}, state 0x{2})",
                this.classId, this.objId, this.state);
        }
    }

    public delegate GameObject FactoryFunc<P>(ref LogicProp<P> prop) where P : unmanaged;
    internal delegate GameObject FactoryFunc(IntPtr ptr);

    public static class SyncAgent {
        private static IntPtr resCache = IntPtr.Zero;
        private static IntPtr syncAgent = IntPtr.Zero;

        private static Dictionary<ClassID, FactoryFunc> factories = new Dictionary<ClassID, FactoryFunc>();
        internal static Dictionary<ObjID, BaseRefState> states = new Dictionary<ObjID, BaseRefState>();
        private static List<ObjID> statesGc = new List<ObjID>();
        private unsafe static DataPool* dataPool = null;

        public static void Load() {
            if (resCache != IntPtr.Zero) {
                throw new Exception("Load()");
            }

            FFI.Load();
            FFI.InitLogger("./critical_point.log");

            resCache = FFI.CreateResCache("./Assets/CriticalPoint/", "resource.yml", "id.yml");
            if (resCache == IntPtr.Zero) {
                throw new Exception("Load()");
            }
        }

        public static void Unload() {
            if (resCache != IntPtr.Zero) {
                FFI.DestoryResCache(resCache);
            }
            FFI.Unload();
        }

        public static void Initialize() {
            if (syncAgent != IntPtr.Zero) {
                throw new Exception("Initialize()");
            }

            syncAgent = FFI.CreateSyncAgent(resCache, 60, "Prefab.Scene.1");
            if (syncAgent == IntPtr.Zero) {
                throw new Exception("CreateSyncAgent()");
            }
        }

        public static void Finialize() {
            if (syncAgent != IntPtr.Zero) {
                FFI.DestorySyncAgent(syncAgent);
                syncAgent = IntPtr.Zero;
            }
        }

        public static void RegisterFactory<P>(FactoryFunc<P> factory) where P : unmanaged {
            var attrs = Attribute.GetCustomAttributes(typeof(P));
            if (attrs.Length == 0) {
                throw new Exception("Invalid Prop type");
            }
            var attr = (DefProp)attrs.FirstOrDefault();
            if (factories.ContainsKey(attr.classId)) {
                throw new Exception(string.Format("Register {0} twice", attr.classId));
            }
            factories.Add(attr.classId, (IntPtr ptr) => {
                unsafe { return factory(ref *(LogicProp<P>*)ptr); };
            });
        }

        public static unsafe void Update() {
            if (syncAgent == IntPtr.Zero) {
                throw new Exception("Update()");
            }

            var newPool = (DataPool*)FFI.SyncAgentUpdate(syncAgent);
            if (newPool == null) {
                throw new Exception("SyncAgentUpdate()");
            }

            DispatchProp((*newPool).props);
            DispatchState((*newPool).states);

            if (dataPool != null) {
                FFI.FreeDataPool((IntPtr)dataPool);
            }
            dataPool = newPool;
        }

        private static List<GameObject> DispatchProp(FFIArray<IntPtr> ptrs) {
            var gos = new List<GameObject>();
            foreach (var ptr in ptrs) {
                if (ptr == IntPtr.Zero) {
                    continue;
                }

                var header = LogicPropHeader.FromPtr(ptr);
                FactoryFunc factory;
                if (!factories.TryGetValue(header.classId, out factory)) {
                    continue;
                }

                var go = factory(ptr);
                if (go == null) {
                    continue;
                }
                gos.Add(go);
            }
            return gos;
        }

        private static void DispatchState(FFIArray<IntPtr> ptrs) {
            foreach (var iter in states.Values) {
                var state = iter;
                var gc = false;
                do {
                    state.SetPtr(IntPtr.Zero);
                    if (state.objId.IsInvalid() && !gc) {
                        statesGc.Add(state.objId);
                        gc = true;
                    }
                } while ((state = state.next) != null);
            }

            foreach (var objId in statesGc) {
                BaseRefState newList = null;
                var state = states[objId];
                do {
                    if (state.objId.IsValid()) {
                        state.next = newList;
                        newList = state;
                    }
                } while ((state = state.next) != null);
                if (newList == null) {
                    states.Remove(objId);
                } else {
                    states[objId] = newList;
                }
            }
            statesGc.Clear();

            foreach (var ptr in ptrs) {
                if (ptr == IntPtr.Zero) {
                    continue;
                }

                var header = LogicStateHeader.FromPtr(ptr);
                BaseRefState state;
                if (!states.TryGetValue(header.objId, out state)) {
                    continue;
                }

                do {
                    if (state.classId != header.classId) {
                        throw new Exception(string.Format("{0} classId != {1}", state, header.classId));
                    }
                    state.SetPtr(ptr);
                } while ((state = state.next) != null);
            }
        }
    }
}
