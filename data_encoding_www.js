let wasm_bindgen = (function(exports) {
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }

    function add_encoding() {
        wasm.add_encoding();
    }
    exports.add_encoding = add_encoding;

    /**
     * @param {number} id
     */
    function delete_encoding(id) {
        wasm.delete_encoding(id);
    }
    exports.delete_encoding = delete_encoding;

    /**
     * @param {string} name
     */
    function goto_tutorial(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.goto_tutorial(ptr0, len0);
    }
    exports.goto_tutorial = goto_tutorial;

    function init() {
        wasm.init();
    }
    exports.init = init;

    /**
     * @param {number} id
     */
    function load_preset(id) {
        wasm.load_preset(id);
    }
    exports.load_preset = load_preset;

    /**
     * @param {number} id
     */
    function move_focus(id) {
        wasm.move_focus(id);
    }
    exports.move_focus = move_focus;

    /**
     * @param {number} id
     */
    function spec_update(id) {
        wasm.spec_update(id);
    }
    exports.spec_update = spec_update;

    /**
     * @param {number} id
     */
    function swap_left(id) {
        wasm.swap_left(id);
    }
    exports.swap_left = swap_left;

    /**
     * @param {number} id
     */
    function swap_right(id) {
        wasm.swap_right(id);
    }
    exports.swap_right = swap_right;

    /**
     * @param {number} id
     */
    function text_update(id) {
        wasm.text_update(id);
    }
    exports.text_update = text_update;

    /**
     * @param {number} id
     */
    function toggle_bit_order(id) {
        wasm.toggle_bit_order(id);
    }
    exports.toggle_bit_order = toggle_bit_order;

    /**
     * @param {string} name
     */
    function toggle_menu(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.toggle_menu(ptr0, len0);
    }
    exports.toggle_menu = toggle_menu;

    /**
     * @param {number} id
     */
    function toggle_trailing_bits(id) {
        wasm.toggle_trailing_bits(id);
    }
    exports.toggle_trailing_bits = toggle_trailing_bits;
    function __wbg_get_imports() {
        const import0 = {
            __proto__: null,
            __wbg___wbindgen_is_null_e343b7d08827ba72: function(arg0) {
                const ret = arg0 === null;
                return ret;
            },
            __wbg___wbindgen_string_get_0380ccaa2f57f0d9: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_addClass_e77042a4a209842d: function(arg0, arg1, arg2) {
                addClass(arg0, getStringFromWasm0(arg1, arg2));
            },
            __wbg_appendChild_4d98b0ed1c29b564: function(arg0, arg1) {
                appendChild(arg0, arg1);
            },
            __wbg_createElement_ebb07dd2fbc66598: function(arg0, arg1) {
                const ret = createElement(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_createTextNode_39c80fa896af307f: function(arg0, arg1) {
                const ret = createTextNode(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_deleteHistory_ba8ebd1f667c42c4: function(arg0, arg1) {
                deleteHistory(getStringFromWasm0(arg0, arg1));
            },
            __wbg_deleteStorage_19ff3f3900c8fca5: function(arg0, arg1) {
                deleteStorage(getStringFromWasm0(arg0, arg1));
            },
            __wbg_focus_1c1c5c609f1a71a5: function(arg0) {
                focus(arg0);
            },
            __wbg_getElementByClass_f573735ef1bcb1c0: function(arg0, arg1, arg2) {
                const ret = getElementByClass(arg0, getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_getElementById_fd1b7deab7889faf: function(arg0, arg1) {
                const ret = getElementById(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_getHistory_c490b7f659ae1cd8: function(arg0, arg1) {
                const ret = getHistory(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_getStorage_6f937108d9166393: function(arg0, arg1) {
                const ret = getStorage(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_hasClass_667d137a4da1f9f8: function(arg0, arg1, arg2) {
                const ret = hasClass(arg0, getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_innerHTML_d644d58c28f7784f: function(arg0, arg1) {
                const ret = innerHTML(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_insertBefore_f2c9c7a42dd65d1e: function(arg0, arg1, arg2) {
                insertBefore(arg0, arg1, arg2);
            },
            __wbg_removeAttribute_777621d5c77745e4: function(arg0, arg1, arg2) {
                removeAttribute(arg0, getStringFromWasm0(arg1, arg2));
            },
            __wbg_removeChild_d756a4874f7ee1d7: function(arg0, arg1) {
                removeChild(arg0, arg1);
            },
            __wbg_removeClass_e16ecd3140fe6f80: function(arg0, arg1, arg2) {
                removeClass(arg0, getStringFromWasm0(arg1, arg2));
            },
            __wbg_setAttribute_cb51c35512b44230: function(arg0, arg1, arg2, arg3, arg4) {
                setAttribute(arg0, getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            },
            __wbg_setHistory_5911ca1fe06abcd9: function(arg0, arg1, arg2, arg3) {
                setHistory(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            },
            __wbg_setStorage_a095000e55d9f86c: function(arg0, arg1, arg2, arg3) {
                setStorage(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            },
            __wbg_set_innerHTML_9f1742c2d23df6a9: function(arg0, arg1, arg2) {
                set_innerHTML(arg0, getStringFromWasm0(arg1, arg2));
            },
            __wbg_set_value_f50f5477da40de51: function(arg0, arg1, arg2) {
                set_value(arg0, getStringFromWasm0(arg1, arg2));
            },
            __wbg_value_b27d4e44f861de20: function(arg0, arg1) {
                const ret = value(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbindgen_init_externref_table: function() {
                const table = wasm.__wbindgen_externrefs;
                const offset = table.grow(4);
                table.set(0, undefined);
                table.set(offset + 0, undefined);
                table.set(offset + 1, null);
                table.set(offset + 2, true);
                table.set(offset + 3, false);
            },
        };
        return {
            __proto__: null,
            "./data_encoding_www_bg.js": import0,
        };
    }

    let cachedDataViewMemory0 = null;
    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        return decodeText(ptr >>> 0, len);
    }

    let cachedUint8ArrayMemory0 = null;
    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    function passStringToWasm0(arg, malloc, realloc) {
        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }
        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    function decodeText(ptr, len) {
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    const cachedTextEncoder = new TextEncoder();

    if (!('encodeInto' in cachedTextEncoder)) {
        cachedTextEncoder.encodeInto = function (arg, view) {
            const buf = cachedTextEncoder.encode(arg);
            view.set(buf);
            return {
                read: arg.length,
                written: buf.length
            };
        };
    }

    let WASM_VECTOR_LEN = 0;

    let wasmModule, wasmInstance, wasm;
    function __wbg_finalize_init(instance, module) {
        wasmInstance = instance;
        wasm = instance.exports;
        wasmModule = module;
        cachedDataViewMemory0 = null;
        cachedUint8ArrayMemory0 = null;
        wasm.__wbindgen_start();
        return wasm;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (!module.ok) {
                throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
            }

            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);
                } catch (e) {
                    const validResponse = expectedResponseType(module.type);

                    if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else { throw e; }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);
        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };
            } else {
                return instance;
            }
        }

        function expectedResponseType(type) {
            switch (type) {
                case 'basic': case 'cors': case 'default': return true;
            }
            return false;
        }
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (module !== undefined) {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();
        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }
        const instance = new WebAssembly.Instance(module, imports);
        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;


        if (module_or_path !== undefined) {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (module_or_path === undefined && script_src !== undefined) {
            module_or_path = script_src.replace(/\.js$/, "_bg.wasm");
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    return Object.assign(__wbg_init, { initSync }, exports);
})({ __proto__: null });
