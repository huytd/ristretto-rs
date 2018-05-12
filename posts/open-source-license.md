# Chuyện giấy phép mã nguồn mở

Có một sự việc khá là đình đám trong thời gian gần đây đó là vấn đề [Giấy phép mã nguồn mở BSD+Patent](https://lwn.net/Articles/728178/) của Facebook dùng trong React và nhiều project khác.

Mọi tranh cãi đều nằm ở file [PATENTS](https://github.com/facebook/react/pull/10804/files#diff-7373d27f0ea94a5b649f893e20fffeda) mà Facebook đã đưa vào project. Mà đỉnh điểm đó là sự kiện Apache tuyên bố đưa giấy phép này vào danh mục Cateogry-X, cấm không cho bất kì dự án nào thuộc Apache Software Foundation sử dụng các project BSD+Patent nữa (trong đó có React và RocksDB).

Rất nhiều dự án lớn đã tuyên bố từ bỏ React, trong đó [có cả WordPress](https://ma.tt/2017/09/on-react-and-wordpress/).

Trước tình thế này thì Facebook đã buộc phải [từ bỏ giấy phép BSD+Patent](https://github.com/facebook/react/pull/10804) trong phiên bản React 16 mới vừa ra mắt hôm qua. Tuy nhiên việc chuyển từ BSD+Patent sang giấy phép MIT cũng không hoàn toàn được lòng tất cả mọi người cho lắm.

<blockquote class="twitter-tweet tw-align-center" data-lang="en"><p lang="en" dir="ltr">The community has decided. ¯\_(ツ)_/¯</p>&mdash; Dan Abramov (@dan_abramov) <a href="https://twitter.com/dan_abramov/status/911354281340612609">September 22, 2017</a></blockquote>
<script async src="//platform.twitter.com/widgets.js" charset="utf-8"></script>

<blockquote class="twitter-tweet tw-align-center" data-lang="en"><p lang="en" dir="ltr">It doesn’t have an explicit grant. I personally think explicitness is a good idea. But I appreciate the simplicity of MIT too.</p>&mdash; Dan Abramov (@dan_abramov) <a href="https://twitter.com/dan_abramov/status/911355333012066305">September 22, 2017</a></blockquote>
<script async src="//platform.twitter.com/widgets.js" charset="utf-8"></script>

Bài viết này không nhằm bình luận hay phán xét gì quyết định của Facebook cũng như ý kiến của các nhân vật trong cộng đồng như Dan hay Ryan. Trong bài này mình chỉ cung cấp một vài thông tin về 2 loại giấy phép BSD và MIT, mình cũng sẽ không đề cập gì thêm đến giấy phép PATENTS.

## Giấy phép BSD

Thực ra cái tên gọi "giấy phép BSD" là chưa đủ, vì có đến 3 loại giấy phép BSD khác nhau: **2-clause BSD**, **3-clause BSD** và **4-clause BSD**.

Nội dung chính của giấy phép **2-clause BSD** gồm có 2 câu như sau (xem [bản gốc tiếng Anh tại đây](https://opensource.org/licenses/BSD-2-Clause)):

```text
Việc phân phối lại và sử dụng dưới dạng source hoặc binary, có hoặc không có sửa đổi, đều được cho phép với các điều kiện sau:

1. Việc phân phối lại source code phải giữ lại các nội dung như đoạn copyright ở trên, danh sách các điều khoản này và phần disclaimer bên dưới.

2. Việc phân phối lại dưới dạng binary phải thể hiện lại các nội dung như copyright ở trên, danh sách các điều khoản này và phần disclaimer bên dưới trong phần tài liệu và/hoặc các tài liệu khác được cung cấp cùng với bản phân phối đó.
```

Giấy phép **3-clause BSD** thêm vào một câu nữa:

```text
3. Tên của [tổ chức] và các contributors đều không được sử dụng để công nhận hoặc quảng bá cho các sản phẩm có nguồn gốc từ phần mềm này, trừ khi được cấp phép dưới dạng văn bản.
```

Giấy phép **4-clause BSD** thêm vào 1 câu nữa:

```text
4. Tất cả mọi hình thức quảng cáo có đề cập đến các chức năng hoặc công dụng của phần mềm này đều phải được hiển thị kèm thông báo sau: Sản phẩm này có bao gồm phần mềm phát triển bởi [tên tổ chức].
```

Như vậy, giấy phép **2-clause BSD** gần như là tự do tuyệt đối cho các nhà phát triển hoặc phân phối chừng nào họ còn tôn trọng các điều khoản của giấy phép. Giấy phép **3-clause BSD** và **4-clause BSD** quy định cụ thể hơn về việc quảng bá, marketing và việc sử dụng danh tính của tổ chức/nhà phát triển gốc cùng toàn bộ các contributors của dự án.

Điều này có nghĩa là nếu bạn có một thư viện mã nguồn mở, bạn công bố nó trên Github với giấy phép 2-clause BSD, thì bất cứ ai cũng có thể tải về và sử dụng, sửa đổi, đưa vào trong dự án của họ, và họ có thể thoải mái sử dụng danh tính của bạn để đưa vào trong việc quảng bá dự án đó. Nếu bạn cảm thấy việc đó nguy hiểm đến uy tín hoặc sự án toàn của bạn (ví dụ một người nào đó sử dụng thư viện bạn làm ra để phát triển một trang web buôn bán ma túy, hoặc đồi trụy, chống phá), bạn có thể sử dụng giấy phép 3-clause BSD để hạn chế việc này. 

Giấy phép mà Facebook sử dụng trước đây là **3-clause BSD**.
