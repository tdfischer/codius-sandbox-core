#include <node.h>
#include <memory>
#include <iostream>
#include <vector>
#include <cstring>
#include <map>

using namespace v8;

extern "C" {
  struct Sandbox;
  extern Sandbox* sandbox_new();
  extern void sandbox_free(Sandbox*);
  extern void sandbox_spawn(Sandbox*, const char** argv);
  extern void sandbox_tick(Sandbox*);
}

class SandboxDeleter {
public:
  void operator()(Sandbox* s) {sandbox_free (s);}
};

class NodeSandbox {
public:
  NodeSandbox();
  ~NodeSandbox();
  static void Init(v8::Handle<v8::Object> exports);

private:
  static v8::Handle<v8::Value> node_spawn(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_kill(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_finish_ipc(const v8::Arguments& args);
  static v8::Handle<v8::Value> node_new(const v8::Arguments& args);
  static v8::Persistent<v8::Function> s_constructor;
  static void cb_signal (uv_signal_t* handle, int signum);
  std::unique_ptr<Sandbox, SandboxDeleter> m_box;
  uv_signal_t m_signal;
};

class SandboxWrapper : public node::ObjectWrap {
  public:
    SandboxWrapper();
    ~SandboxWrapper();
    std::unique_ptr<NodeSandbox> sbox;
    friend class NodeSandbox;
};


extern "C" {
  void this_is_a_rust_function_for_c_api();
}

SandboxWrapper::SandboxWrapper()
  : sbox (new NodeSandbox())
{}

SandboxWrapper::~SandboxWrapper()
{}

NodeSandbox::NodeSandbox()
  : m_box (sandbox_new())
{
  uv_signal_init (uv_default_loop(), &m_signal);
  m_signal.data = this;
}

NodeSandbox::~NodeSandbox()
{
  std::cout << "Deleted";
}

Handle<Value>
NodeSandbox::node_new(const Arguments& args)
{
  HandleScope scope;

  if (args.IsConstructCall()) {
    SandboxWrapper* wrap = new SandboxWrapper();
    wrap->Wrap(args.This());
    node::MakeCallback (wrap->handle_, "_init", 0, nullptr);
    //wrap->handle_->SetAccessor (String::NewSymbol ("debuggerOnCrash"), NodeSandbox::node_getDebugOnCrash, NodeSandbox::node_setDebugOnCrash);
    /*wrap->sbox->addIPC (std::unique_ptr<NodeIPC> (new NodeIPC (STDOUT_FILENO, wrap->handle_)));
    wrap->sbox->addIPC (std::unique_ptr<NodeIPC> (new NodeIPC (STDERR_FILENO, wrap->handle_)));*/

    return args.This();
  } else {
    Local<Value> argv[1] = { args[0] };
    return scope.Close(s_constructor->NewInstance(1, argv));
  }
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

void
NodeSandbox::cb_signal (uv_signal_t* handle, int signum)
{
  NodeSandbox* sbox = (NodeSandbox*)handle->data;
  sandbox_tick (sbox->m_box.get());
}

Handle<Value>
NodeSandbox::node_spawn(const Arguments& args)
{
  HandleScope scope;
  std::vector<std::string> argv (args.Length());
  std::map<std::string, std::string> envp;
  SandboxWrapper* wrap;
  std::vector<const char*> argv_ptrs;

  wrap = node::ObjectWrap::Unwrap<SandboxWrapper>(args.This());

  for(int i = 0; i < args.Length(); i++) {
    if (args[i]->IsString()) {
      Local<String> v = args[i]->ToString();
      argv[i].reserve (v->Utf8Length() + 1);
      v->WriteUtf8(const_cast<char*> (argv[i].c_str()));
    } else {
      if (i <= args.Length() - 2 ) {
        ThrowException(Exception::TypeError(String::New("Arguments must be strings.")));
        goto out;
      } else {
        // Last parameter is an options structure
        Local<Object> options = args[i]->ToObject();
        if (!options.IsEmpty()) {
          if (options->HasRealNamedProperty(String::NewSymbol("env"))) {
            Local<Object> envOptions = options->Get(String::NewSymbol("env"))->ToObject();
            if (!envOptions.IsEmpty()) {
              Local<Array> envArray = envOptions->GetOwnPropertyNames();
              for (uint32_t i = 0; i < envArray->Length(); i++) {
                std::vector<char> strName;
                std::vector<char> strValue;
                Local<String> envName;
                Local<String> envValue;

                if (!(envArray->Get(i)->IsString() && envArray->Get(i)->IsString()))
                  goto err_env;

                envName = envArray->Get(i)->ToString();
                envValue = envOptions->Get(envName)->ToString();

                strName.resize (envName->Utf8Length()+1);
                strValue.resize (envValue->Utf8Length()+1);
                envName->WriteUtf8 (strName.data());
                envValue->WriteUtf8 (strValue.data());
                envp.insert (std::make_pair(std::string (strName.data()), std::string(strValue.data())));
              }
            } else {
              goto err_env;
            }
          }
        } else {
          goto err_options;
        }
      }
    }
  }



  //wrap->sbox->getVFS().setCWD ("/contract/");
  //wrap->sbox->spawn(argv, envp);
  for (unsigned int i = 0; i < argv.size()-1; i++) {
    argv_ptrs.push_back (strdup (argv[i].c_str()));
  }
  argv_ptrs.push_back (0);
  std::cout << "Pointer count: " << argv_ptrs.size() << std::endl;

  sandbox_spawn (wrap->sbox->m_box.get(), argv_ptrs.data());
  uv_signal_start (&wrap->sbox->m_signal, cb_signal, SIGCHLD);

  goto out;

err_env:
  ThrowException(Exception::TypeError(String::New("'env' option must be a map of string:string")));
  goto out;

err_options:
  ThrowException(Exception::TypeError(String::New("Last argument must be an options structure.")));
  goto out;

out:

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
