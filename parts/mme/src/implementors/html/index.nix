{ mize_url, ... }: ''
<html>
  <head>
    <script src="${mize_url}"></script>
    <script>
      init_mize({
        load_modules: "mme",
        mod: {
          mme: {
            webview_con: true
          }
        }
      })
    </script>
  </head>
	<body>
  	hello world...............
  </body>  
</html>
''
