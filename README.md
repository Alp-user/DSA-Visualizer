# Veri Yapıları Görselleştirici

C++ programlarındaki veri yapılarını ve bu yapıların program içerisindeki değişikliklerini adım adım takip edebilmeyi sağlayan bir kütüphane.

## Motivasyon

İnternette veri yapılarını görselleştirmek için araçlar mevcut, ancak araştırdığım bütün araçlarda ya önemli eksiklikler var ya da aşağıdaki özellikleri sunmuyorlar:

- **Esneklik**: Yazılımcının kodundaki her türlü özel veri yapısıyla çalışabilmeli
- **Kolay Kurulum**: Minimal kod değişikliği gerektirmeli
- **Adım Adım İzleme**: Veri yapılarındaki değişiklikler zaman içinde takip edilebilmeli
- **Hata Ayıklama**: Program beklenmedik şekilde sonlansa bile yapılar kaydedilebilmeli

## Özellikler

### Piyasadaki Araçlardan Farkları

- Yazılımcı tarafından tasarlanmış özel veri yapılarıyla çalışır
- Kolay entegrasyon: Sadece birkaç satır kod değişikliği yeterli
- **Değişiklikleri takip eder**: Diğer araçlar sadece anlık durumu gösterirken, bu kütüphane zaman içindeki değişiklikleri kaydeder

## Nasıl Çalışır?

### 1. AbstractNode Sınıfı

Her türlü veri yapısını ortak bir arayüz altında birleştiren abstract virtual class:

```cpp
class AbstractNode{
public:
  unsigned int node_id;
  static unsigned int global_id;
  
  AbstractNode(): node_id(global_id++){
    JsonManager::add_node_graph_json(node_id, "");
  }
  
  virtual string stringify() const{
    return std::to_string(node_id);
  }
  
  virtual string weightify(const AbstractNode* other_end) const{
    return "";
  }

  virtual vector<const AbstractNode*> nexts() const = 0;

  virtual ~AbstractNode() = default;
};
```

### 2. Kendi Veri Yapınızı Tanımlayın

Sadece `AbstractNode`'u inherit edin ve gerekli fonksiyonları tanımlayın:

```cpp
template<typename T>
struct BSTNode : public AbstractNode {
  T data;
  BSTNode* left;
  BSTNode* right;
  
  BSTNode(const T& v): data(v), left(nullptr), right(nullptr) {}
  
  virtual vector<const AbstractNode*> nexts() const override {
    vector<const AbstractNode*> res;
    if(left) res.push_back(left);
    if(right) res.push_back(right);
    return res;
  }
  
  virtual string stringify() const override {
    return std::to_string(data);
  }
};
```

### 3. Listener ile Değişiklikleri Takip Edin

```cpp
int main(){
  binary_search_tree<int> bst;
  Listener first_listener;
  
  bst.insert(6, first_listener, "");
  bst.insert(5, first_listener, "");
  bst.insert(7, first_listener, "");
  // ...
}
```

Veri yapınızdaki fonksiyonlara listener ekleyin:

```cpp
void insert(const T& value, Listener& listener, const char* note = "insert"){
  root = insert_node(root, value);
  listener.graph_listen(root, note);
}
```

## Listener Nasıl Çalışır?

`Listener` sınıfı her `graph_listen` çağrısında:

1. Önceki durum ile şu anki durumu karşılaştırır
2. Farklılıkları 6 kategoride kaydeder:
   - Eklenen node'lar
   - Çıkarılan node'lar
   - Eklenen edge'ler
   - Çıkarılan edge'ler
   - Değişen node verileri
   - Değişen weight verileri
3. Tüm değişiklikleri JSON formatında saklar
4. Program beklenmedik kapanırsa bile JSON dosyasını kaydetmeyi garanti eder

## Örnek Kullanım

```cpp
binary_search_tree<int> bst;
Listener first_listener;

bst.insert(6, first_listener, "");
bst.insert(5, first_listener, "");
bst.insert(7, first_listener, "");
bst.insert(2, first_listener, "");
// ... daha fazla işlem

bst.remove(1, first_listener, "");
bst.remove(6, first_listener, "");
// ...
```

## Kullanım Alanları

Bu kütüphane özellikle veri yapıları eğitiminde faydalıdır. Öğrenciler kendi yazdıkları veri yapılarının nasıl çalıştığını görsel olarak takip edebilir ve hataları kolayca tespit edebilirler.

## Teknik Detaylar

- **Dil**: C++ ve Rust
- **Görselleştirme**: OpenGL
- **Veri Formatı**: JSON
