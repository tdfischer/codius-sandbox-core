#include <node.h>

using namespace v8;

extern "C" {
  extern void* sandbox_new();
}

class NodeSandbox {
public:
  NodeSandbox();
  static void Init(v8::Handle<v8::Object> exports);

private:
  static v8::Handle<v8::Value> node_spawn(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_kill(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_finish_ipc(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_new(const v8::Arguments& args);
  static v8::Persistent<v8::Function> s_constructor;
};

extern "C" {
  void this_is_a_rust_function_for_c_api();
}

Handle<Value>
NodeSandbox::node_new(const Arguments& args)
{
  this_is_a_rust_function_for_c_api();
  return Undefined();
}

Handle<Value>
NodeSandbox::node_kill(const Arguments& args)
{
  return Undefined();
}

Handle<Value>
NodeSandbox::node_finish_ipc(const Arguments& args)
{
  return Undefined();
}

Handle<Value>
NodeSandbox::node_spawn(const Arguments& args)
{
  return Undefined();
}

Persistent<Function> NodeSandbox::s_constructor;

void
NodeSandbox::Init(Handle<Object> exports) {
  Local<FunctionTemplate> tpl = FunctionTemplate::New(node_new);
  tpl->SetClassName(String::NewSymbol("Sandbox"));
  tpl->InstanceTemplate()->SetInternalFieldCount(2);
  node::SetPrototypeMethod(tpl, "spawn", node_spawn);
  node::SetPrototypeMethod(tpl, "kill", node_kill);
  node::SetPrototypeMethod(tpl, "finishIPC", node_finish_ipc);
  s_constructor = Persistent<Function>::New(tpl->GetFunction());
  exports->Set(String::NewSymbol("Sandbox"), s_constructor);
}

void
init(Handle<Object> exports) {
  NodeSandbox::Init(exports);
}

NODE_MODULE (node_codius_sandbox, init);
